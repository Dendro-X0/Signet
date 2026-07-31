use std::path::PathBuf;

use clap::Args as ClapArgs;

use crate::identity::load_active;
use crate::project::ProjectCtx;
use crate::sign::{
    discover_artifacts, find_tauri_cli, host_signable, resolve_src_tauri, sign_host_artifacts,
    tool_hint_for_host, write_sha256sums, DiscoveredArtifact, SignOptions,
};

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to selfsign.toml
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Skip `tauri build`; only discover + sign existing artifacts
    #[arg(long)]
    pub skip_build: bool,

    /// Build without signing (still writes SHA256SUMS when artifacts exist)
    #[arg(long)]
    pub no_sign: bool,

    /// Cargo/Tauri profile (default: release)
    #[arg(long, default_value = "release")]
    pub profile: String,

    /// Do not request an Authenticode timestamp (Windows)
    #[arg(long)]
    pub no_timestamp: bool,

    /// Extra args forwarded to `tauri build` (repeatable)
    #[arg(long = "tauri-arg")]
    pub tauri_args: Vec<String>,

    /// Explicit artifact paths to sign (skips discovery when set)
    #[arg(long = "artifact")]
    pub artifacts: Vec<PathBuf>,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let ctx = ProjectCtx::load(args.config.as_deref()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `selfsign init` in your Tauri app directory first")
    })?;

    let src_tauri = resolve_src_tauri(&ctx.root, &ctx.config.project.tauri_root);
    println!("tauri crate: {}", src_tauri.display());

    if !args.skip_build {
        let Some(cli) = find_tauri_cli() else {
            anyhow::bail!(
                "tauri CLI not found — install with: cargo install tauri-cli --version \"^2\"\n\
                 or pass --skip-build to sign existing artifacts only"
            );
        };
        println!("running tauri build ({})…", args.profile);
        // Note: profile selection is typically via cargo features / tauri config;
        // we forward extra args and run standard `tauri build`.
        cli.run_build(&src_tauri, &args.tauri_args)?;
    } else {
        println!("skipping tauri build (--skip-build)");
    }

    let discovered = if args.artifacts.is_empty() {
        discover_artifacts(&src_tauri, &args.profile)?
    } else {
        args.artifacts
            .iter()
            .map(|p| DiscoveredArtifact {
                path: p.clone(),
                kind: classify_explicit(p),
            })
            .collect()
    };

    if discovered.is_empty() {
        anyhow::bail!(
            "no artifacts found under {}/target/{}/bundle\n\
             hint: run a successful `tauri build` first, or pass --artifact <path>",
            src_tauri.display(),
            args.profile
        );
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
    }

    if args.no_sign {
        println!("signing skipped (--no-sign)");
        println!(
            "note: OS warnings (SmartScreen / Gatekeeper) are unchanged by checksums alone"
        );
        return Ok(());
    }

    let identity = load_active(&ctx.identity_root()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `selfsign identity create` first (or pass --no-sign)")
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
    }

    println!("done — compare fingerprints via `selfsign identity show` / TRUST.md");
    Ok(())
}

fn classify_explicit(path: &std::path::Path) -> crate::sign::ArtifactKind {
    use crate::sign::ArtifactKind;
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if name.ends_with(".appimage") {
        return ArtifactKind::LinuxAppImage;
    }
    if path.extension().and_then(|e| e.to_str()) == Some("app") || name.ends_with(".app") {
        return ArtifactKind::MacApp;
    }
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "exe" => ArtifactKind::WindowsExe,
        "msi" | "msix" => ArtifactKind::WindowsMsi,
        "dmg" => ArtifactKind::MacDmg,
        "deb" => ArtifactKind::LinuxDeb,
        "rpm" => ArtifactKind::LinuxRpm,
        _ => ArtifactKind::Other,
    }
}
