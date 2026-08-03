use std::path::PathBuf;

use clap::Args as ClapArgs;

use crate::android::{keystore_paths, sign_apks};
use crate::artifact::{select_adapter, Artifact, ArtifactKind, BuildOpts};
use crate::config::select_targets;
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

    /// Build only this `[[targets]].id` (default: all targets)
    #[arg(long)]
    pub target: Option<String>,

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

    let all = ctx.targets();
    let selected = select_targets(&all, args.target.as_deref())?;

    let mut discovered: Vec<Artifact> = Vec::new();
    let mut saw_ios = false;

    if args.artifacts.is_empty() {
        for target in &selected {
            println!("── target {} ({}) ──", target.id, target.framework);
            let tctx = ctx.with_target(target);
            let adapter = select_adapter(&tctx.root, &tctx.config)?;
            let label = adapter.label_root(&tctx);
            println!("{} root: {}", adapter.id(), label.display());

            if !args.skip_build {
                adapter.build(
                    &tctx,
                    &BuildOpts {
                        profile: args.profile.clone(),
                        extra_args: args.tauri_args.clone(),
                    },
                )?;
            } else {
                println!("skipping {} build (--skip-build)", adapter.id());
            }

            if adapter.id() == "ios" {
                saw_ios = true;
            }

            for art in adapter.discover(&tctx, &args.profile)? {
                if !discovered.iter().any(|a| a.path == art.path) {
                    discovered.push(art);
                }
            }
        }
    } else {
        println!("using {} explicit --artifact path(s)", args.artifacts.len());
        discovered = args.artifacts.iter().map(Artifact::from_path).collect();
    }

    if discovered.is_empty() {
        let hints: Vec<String> = selected
            .iter()
            .map(|t| {
                let tctx = ctx.with_target(t);
                select_adapter(&tctx.root, &tctx.config)
                    .map(|a| a.empty_hint(&tctx, &args.profile))
                    .unwrap_or_else(|e| e.to_string())
            })
            .collect();
        anyhow::bail!("{}", hints.join("\n"));
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

    let apks: Vec<Artifact> = discovered
        .iter()
        .filter(|a| a.kind == ArtifactKind::Apk)
        .cloned()
        .collect();
    let host = host_signable(&discovered);

    if saw_ios || discovered.iter().any(|a| a.kind == ArtifactKind::Ipa) {
        println!("note: {}", crate::ios::honesty_notes());
        println!(
            "note: IPA/device signing stays in Xcode — `signet ios package --app …` for IPA layout"
        );
    }

    if !apks.is_empty() {
        sign_android_flow(&ctx, &apks, &checksum_paths, &sums_path, &args)?;
    }

    if host.is_empty() && apks.is_empty() {
        if saw_ios || discovered.iter().any(|a| a.kind == ArtifactKind::Ipa) {
            println!("done — see docs/ios.md");
            return Ok(());
        }
        println!("nothing to host-sign (no host-matching or APK artifacts)");
        return Ok(());
    }

    if host.is_empty() {
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

    let opts = SignOptions {
        timestamp: !args.no_timestamp,
        ..SignOptions::default()
    };
    let report = sign_host_artifacts(&identity, &host, &opts)?;

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

    let mut refresh = checksum_paths;
    for s in &report.signed {
        if s.path.is_file() && !refresh.iter().any(|p| p == &s.path) {
            refresh.push(s.path.clone());
        }
    }
    if !refresh.is_empty() {
        write_sha256sums(&sums_path, &refresh)?;
        println!("wrote {} (post-sign)", sums_path.display());
        emit_sums_sign(&ctx, &sums_path, &args)?;
    }

    println!("done — compare fingerprints via `signet identity show` / TRUST.md");
    Ok(())
}

fn sign_android_flow(
    ctx: &ProjectCtx,
    apk_arts: &[Artifact],
    checksum_paths: &[PathBuf],
    sums_path: &std::path::Path,
    args: &Args,
) -> anyhow::Result<()> {
    let paths = keystore_paths(&ctx.secrets_dir());
    let apks: Vec<PathBuf> = apk_arts.iter().map(|a| a.path.clone()).collect();
    if apks.is_empty() {
        return Ok(());
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
        println!("wrote {} (post-sign)", sums_path.display());
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
