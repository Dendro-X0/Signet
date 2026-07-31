use std::path::PathBuf;
use std::process::Command;

use clap::Args as ClapArgs;

use crate::identity::load_active;
use crate::project::ProjectCtx;
use crate::release::{
    build_release_notes, collect_release_files, detect_github_repo, publish_github_release,
    verify_checksums_cover, GitHubPublishOpts,
};

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to selfsign.toml
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Git tag / version to publish (required unless --dry-run with discovery only)
    #[arg(long)]
    pub tag: Option<String>,

    /// GitHub repo `owner/name` (overrides config / git remote)
    #[arg(long)]
    pub repo: Option<String>,

    /// Cargo/Tauri profile used to locate bundles
    #[arg(long, default_value = "release")]
    pub profile: String,

    /// Compute checksums and list assets; do not upload
    #[arg(long)]
    pub dry_run: bool,

    /// Create the GitHub Release as a draft
    #[arg(long)]
    pub draft: bool,

    /// Mark the GitHub Release as prerelease
    #[arg(long)]
    pub prerelease: bool,

    /// Do not replace existing assets when the tag already has a release
    #[arg(long)]
    pub no_clobber: bool,

    /// Do not attach TRUST.md even if present
    #[arg(long)]
    pub no_trust: bool,

    /// Release title (default: tag)
    #[arg(long)]
    pub title: Option<String>,

    /// Explicit artifact paths (skips bundle discovery when set)
    #[arg(long = "artifact")]
    pub artifacts: Vec<PathBuf>,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let ctx = ProjectCtx::load(args.config.as_deref()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `selfsign init` in your Tauri app directory first")
    })?;

    if !ctx.config.release.github && !args.dry_run {
        anyhow::bail!("[release] github = false in selfsign.toml — enable it or use --dry-run");
    }

    let attach_trust = ctx.config.release.attach_trust && !args.no_trust;
    let files = collect_release_files(
        &ctx.root,
        &ctx.config,
        &args.profile,
        &args.artifacts,
        attach_trust,
    )?;

    if files.is_empty() {
        anyhow::bail!(
            "no release assets found — run `selfsign build` first or pass --artifact <path>"
        );
    }

    verify_checksums_cover(&files)?;

    println!("release assets ({}):", files.len());
    for f in &files {
        println!("  [{:<10}] {} ({})", f.kind, f.asset_name, f.path.display());
    }

    let identity = load_active(&ctx.identity_root()).ok();
    let tag = match args.tag.clone().or_else(detect_tag_hint) {
        Some(t) => t,
        None if args.dry_run => "v0.0.0-dry-run".into(),
        None => anyhow::bail!("missing --tag (e.g. --tag v1.0.0)"),
    };
    let title = args.title.clone().unwrap_or_else(|| tag.clone());
    let trust_attached = files.iter().any(|f| f.asset_name == "TRUST.md");
    let notes = build_release_notes(&ctx.config, &tag, identity.as_ref(), trust_attached);

    if args.dry_run {
        let repo = detect_github_repo(
            args.repo.as_deref(),
            &ctx.config.release.repo,
            &ctx.root,
        )
        .unwrap_or_else(|_| "(undetected — set --repo or [release].repo)".into());
        println!("\ndry-run: would publish tag '{tag}' to {repo}");
        println!("title: {title}");
        println!("--- notes ---");
        println!("{notes}");
        return Ok(());
    }

    let repo = detect_github_repo(args.repo.as_deref(), &ctx.config.release.repo, &ctx.root)?;
    println!("publishing {tag} → {repo}");

    let result = publish_github_release(
        &GitHubPublishOpts {
            repo,
            tag,
            name: title,
            notes,
            draft: args.draft,
            prerelease: args.prerelease,
            clobber: !args.no_clobber,
        },
        &files,
    )?;

    println!("published via {} — {}", result.method, result.url);
    Ok(())
}

fn detect_tag_hint() -> Option<String> {
    let out = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let tag = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if tag.is_empty() {
        None
    } else {
        Some(tag)
    }
}
