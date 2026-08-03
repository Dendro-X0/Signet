use std::path::PathBuf;

use clap::Args as ClapArgs;

use crate::android::{keystore_paths, sign_apks};
use crate::artifact::{select_adapter, Artifact, ArtifactKind, BuildOpts};
use crate::identity::load_active;
use crate::project::ProjectCtx;
use crate::sign::{
    host_signable, maybe_sign_sums, sign_host_artifacts, tool_hint_for_host, write_sha256sums,
    SignOptions,
};

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to signet.toml
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Skip framework build; only discover + sign existing artifacts
    #[arg(long)]
    pub skip_build: bool,

    /// Build without signing (still writes SHA256SUMS when artifacts exist)
    #[arg(long)]
    pub no_sign: bool,

    /// Skip minisign/GPG signing of SHA256SUMS
    #[arg(long)]
    pub no_sums_sign: bool,

    /// Fail if minisign cannot sign SHA256SUMS
    #[arg(long)]
    pub require_sums_sign: bool,

    /// Fail if GPG checksum signing was requested but did not succeed
    #[arg(long)]
    pub require_gpg: bool,

    /// Cargo/Tauri profile (default: release)
    #[arg(long, default_value = "release")]
    pub profile: String,

    /// Do not request an Authenticode timestamp (Windows)
    #[arg(long)]
    pub no_timestamp: bool,

    /// Extra args forwarded to the framework build (Tauri: `tauri build`)
    #[arg(long = "tauri-arg")]
    pub tauri_args: Vec<String>,

    /// Explicit artifact paths to sign (skips discovery when set)
    #[arg(long = "artifact")]
    pub artifacts: Vec<PathBuf>,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let ctx = ProjectCtx::load(args.config.as_deref()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `signet init` in your app directory first")
    })?;

    let adapter = select_adapter(&ctx.root, &ctx.config)?;
    let label = adapter.label_root(&ctx);
    println!("{} root: {}", adapter.id(), label.display());

    if !args.skip_build {
        adapter.build(
            &ctx,
            &BuildOpts {
                profile: args.profile.clone(),
                extra_args: args.tauri_args.clone(),
            },
        )?;
    } else {
        println!("skipping {} build (--skip-build)", adapter.id());
    }

    let discovered = if args.artifacts.is_empty() {
        adapter.discover(&ctx, &args.profile)?
    } else {
        args.artifacts.iter().map(Artifact::from_path).collect()
    };

    if discovered.is_empty() {
        anyhow::bail!("{}", adapter.empty_hint(&ctx, &args.profile));
    }

    println!("discovered {} artifact(s):", discovered.len());
    for a in &discovered {
        println!("  [{}] {}", a.kind.as_str(), a.path.display());
    }

    let checksum_paths: Vec<PathBuf> = discovered
        .iter()
        .filter(|a| a.path.is_file())
        .map(|a| a.path.clone())
        .collect();
    let sums_path = ctx.root.join("SHA256SUMS");
    if !checksum_paths.is_empty() {
        write_sha256sums(&sums_path, &checksum_paths)?;
        println!("wrote {}", sums_path.display());
        emit_sums_sign(&ctx, &sums_path, &args)?;
    }

    if args.no_sign {
        println!("signing skipped (--no-sign)");
        println!(
            "note: OS warnings (SmartScreen / Gatekeeper) are unchanged by checksums alone"
        );
        return Ok(());
    }

    let is_android = adapter.id() == "android";
    if is_android {
        return sign_android_flow(&ctx, &discovered, &checksum_paths, &sums_path, &args);
    }

    if adapter.id() == "ios" {
        println!("note: {}", crate::ios::honesty_notes());
        if !args.no_sign {
            println!(
                "signing skipped for framework=ios — use Xcode for device signing; \
                 `signet ios package --app …` for IPA layout"
            );
        }
        println!("done — see docs/ios.md");
        return Ok(());
    }

    let identity = load_active(&ctx.identity_root()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `signet identity create` first (or pass --no-sign)")
    })?;
    println!(
        "signing with identity '{}' ({})",
        identity.meta.name, identity.meta.fingerprint_sha256
    );
    println!("host tools: {}", tool_hint_for_host());

    let to_sign = if args.artifacts.is_empty() {
        host_signable(&discovered)
    } else {
        discovered.clone()
    };

    let opts = SignOptions {
        timestamp: !args.no_timestamp,
        ..SignOptions::default()
    };
    let report = sign_host_artifacts(&identity, &to_sign, &opts)?;

    for w in &report.warnings {
        println!("warning: {w}");
    }
    for s in &report.signed {
        println!("signed: {} ({})", s.path.display(), s.method);
        if let Some(note) = &s.note {
            println!("  note: {note}");
        }
    }
    for (path, reason) in &report.skipped {
        println!("skipped: {} — {reason}", path.display());
    }

    if report.signed.is_empty() && report.skipped.is_empty() {
        println!("nothing signed (no host-matching artifacts)");
    } else if report.signed.is_empty() {
        anyhow::bail!("signing produced no successful artifacts");
    }

    // Refresh checksums after signing (signature bytes may change PE checksum overlays;
    // detached .sig files are separate).
    let mut refresh = checksum_paths;
    for s in &report.signed {
        if s.path.is_file() && !refresh.iter().any(|p| p == &s.path) {
            refresh.push(s.path.clone());
        }
    }
    if !refresh.is_empty() {
        write_sha256sums(&sums_path, &refresh)?;
        emit_sums_sign(&ctx, &sums_path, &args)?;
    }

    println!("done — compare fingerprints via `signet identity show` / TRUST.md");
    Ok(())
}

fn sign_android_flow(
    ctx: &ProjectCtx,
    discovered: &[Artifact],
    checksum_paths: &[PathBuf],
    sums_path: &std::path::Path,
    args: &Args,
) -> anyhow::Result<()> {
    let paths = keystore_paths(&ctx.secrets_dir());
    let apks: Vec<PathBuf> = discovered
        .iter()
        .filter(|a| a.kind == ArtifactKind::Apk)
        .map(|a| a.path.clone())
        .collect();
    if apks.is_empty() {
        anyhow::bail!(
            "no APKs to sign — pass --artifact *.apk or build an APK first\n\
             note: AAB Play upload is documented in docs/android.md (not auto-signed as Play distribution)"
        );
    }
    println!(
        "signing {} APK(s) with android keystore ({})",
        apks.len(),
        paths.keystore.display()
    );
    println!("note: local keystore ≠ Play App Signing key — see docs/android.md");

    let report = sign_apks(&paths, &apks)?;
    for w in &report.warnings {
        println!("warning: {w}");
    }
    for (p, method) in &report.signed {
        println!("signed: {} ({method})", p.display());
    }
    for (p, reason) in &report.skipped {
        println!("skipped: {} — {reason}", p.display());
    }
    if report.signed.is_empty() {
        anyhow::bail!("android signing produced no successful APKs");
    }

    let mut refresh = checksum_paths.to_vec();
    for (p, _) in &report.signed {
        if p.is_file() && !refresh.iter().any(|x| x == p) {
            refresh.push(p.clone());
        }
    }
    if !refresh.is_empty() {
        write_sha256sums(sums_path, &refresh)?;
        emit_sums_sign(ctx, sums_path, args)?;
    }
    println!("done — compare Android cert digest via `signet android keystore show` / TRUST.md");
    Ok(())
}

fn emit_sums_sign(
    ctx: &ProjectCtx,
    sums_path: &std::path::Path,
    args: &Args,
) -> anyhow::Result<()> {
    let report = maybe_sign_sums(
        sums_path,
        &ctx.secrets_dir(),
        &ctx.config.trust.checksum_signing,
        args.no_sums_sign,
        args.require_sums_sign,
        args.require_gpg,
    )?;
    for w in &report.warnings {
        println!("warning: {w}");
    }
    if let Some(p) = &report.minisig {
        println!("wrote {}", p.display());
    }
    if let Some(p) = &report.asc {
        println!("wrote {}", p.display());
    }
    Ok(())
}
