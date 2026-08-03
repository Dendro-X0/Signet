use std::path::PathBuf;

use clap::Args as ClapArgs;

use crate::identity::load_active;
use crate::project::ProjectCtx;
use crate::release::{
    build_release_notes, collect_release_files_with_opts, detect_github_repo,
    publish_github_release, CollectOpts, GitHubPublishOpts,
};

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to signet.toml
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

    /// Allow publish when declared [platforms] are missing on-disk artifacts
    #[arg(long)]
    pub allow_partial: bool,

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

    /// Skip minisign/GPG signing of SHA256SUMS
    #[arg(long)]
    pub no_sums_sign: bool,

    /// Fail if minisign cannot sign SHA256SUMS
    #[arg(long)]
    pub require_sums_sign: bool,

    /// Fail if GPG checksum signing was requested but did not succeed
    #[arg(long)]
    pub require_gpg: bool,

    /// Release title (default: tag)
    #[arg(long)]
    pub title: Option<String>,

    /// Explicit artifact paths (skips bundle discovery when set)
    #[arg(long = "artifact")]
    pub artifacts: Vec<PathBuf>,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let ctx = ProjectCtx::load(args.config.as_deref()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `signet init` in your project directory first")
    })?;

    if !ctx.config.release.github && !args.dry_run {
        anyhow::bail!("[release] github = false in signet.toml — enable it or use --dry-run");
    }

    let attach_trust = ctx.config.release.attach_trust && !args.no_trust;
    let files = collect_release_files_with_opts(
        &ctx.root,
        &ctx.config,
        &args.profile,
        &args.artifacts,
        attach_trust,
        CollectOpts {
            no_sums_sign: args.no_sums_sign,
            require_sums_sign: args.require_sums_sign,
            require_gpg: args.require_gpg,
            read_only: args.dry_run,
        },
    )?;

    if files.is_empty() {
        anyhow::bail!(
            "no release assets found — run `signet build` / `signet ship --collect` first or pass --artifact <path>"
        );
    }

    let coverage = crate::ship::assess_coverage(&ctx.root, &ctx.config);
    if coverage.has_gap() {
        let gap = coverage.gap.join(", ");
        if args.dry_run {
            eprintln!(
                "warning: ship coverage gap [{}] — {}",
                gap,
                coverage.summary_line()
            );
            eprintln!("warning: live release will fail without --allow-partial until gap is filled");
        } else if !args.allow_partial {
            anyhow::bail!(
                "ship coverage gap [{gap}] — declared [platforms] missing artifacts.\n\
                 Collect with `signet ship --collect DIR` or pass --allow-partial.\n\
                 {}",
                coverage.summary_line()
            );
        } else {
            eprintln!(
                "warning: --allow-partial: publishing with coverage gap [{gap}]"
            );
        }
    }

    crate::release::verify_checksums_cover_opts(&files, args.dry_run)?;

    println!("release assets ({}):", files.len());
    for f in &files {
        println!("  [{:<10}] {} ({})", f.kind, f.asset_name, f.path.display());
    }

    let identity = load_active(&ctx.identity_root()).ok();
    let suggested = crate::version_detect::default_release_tag(&ctx.root);
    let tag = match &args.tag {
        Some(t) => t.clone(),
        None => {
            println!("using tag '{suggested}' (from project version / git; pass --tag to override)");
            suggested
        }
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
        println!(
            "note: dry-run is read-only — SHA256SUMS not rewritten (live release flattens to asset basenames)"
        );
        let arts: Vec<_> = files
            .iter()
            .filter(|f| !matches!(f.kind, "checksums" | "checksums-sig" | "trust"))
            .map(|f| crate::artifact::Artifact::from_path(&f.path))
            .collect();
        println!("artifacts_json: {}", crate::artifact::artifacts_json(&arts));
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
