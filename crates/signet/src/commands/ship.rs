//! `signet ship` — coverage plan, CI workflow emit, multi-host collect, CI secrets.

use std::fs;
use std::path::PathBuf;

use clap::{Args as ClapArgs, Subcommand};

use crate::project::ProjectCtx;
use crate::ship::{
    assess_ci_readiness, assess_coverage, assess_sign_profile, collect_into_staging, run_secrets,
    render_signet_ship_workflow, workflow_rel_path, SecretsArgs,
};
use crate::ui::console;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to signet.toml
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub cmd: Option<ShipCmd>,

    /// Print declared vs present platform coverage
    #[arg(long, default_value_t = true)]
    pub plan: bool,

    /// Write `.github/workflows/signet-ship.yml` for declared platforms
    #[arg(long)]
    pub ci: bool,

    /// Merge installers from DIR into dist/signet-ship/ and rewrite SHA256SUMS
    #[arg(long, value_name = "DIR")]
    pub collect: Option<PathBuf>,

    /// Overwrite existing workflow when using --ci
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Subcommand)]
pub enum ShipCmd {
    /// Assess / push GitHub Actions secrets from local `.signet/` (dry-run by default)
    Secrets(SecretsCli),
}

#[derive(Debug, ClapArgs)]
pub struct SecretsCli {
    /// Print or apply `gh secret set` recipe
    #[arg(long)]
    pub push: bool,

    /// Actually run `gh secret set` (requires --push)
    #[arg(long)]
    pub apply: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let ctx = ProjectCtx::load(args.config.as_deref()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `signet init` in your app directory first")
    })?;

    if let Some(ShipCmd::Secrets(s)) = args.cmd {
        return run_secrets(
            &ctx,
            SecretsArgs {
                push: s.push,
                apply: s.apply,
            },
        );
    }

    if args.ci {
        return emit_ci(&ctx, args.force);
    }
    if let Some(dir) = args.collect {
        return run_collect(&ctx, &dir);
    }

    let _ = args.plan;
    let report = assess_coverage(&ctx.root, &ctx.config);
    let profile = assess_sign_profile(&ctx.config);
    let ci = assess_ci_readiness(&ctx.root, &ctx.config);

    console::banner("ship · plan");
    report.print_human();
    console::blank();
    profile.print_human();
    console::blank();
    ci.print_human();
    console::blank();
    console::note(&report.summary_line());
    console::note(&profile.summary_line());
    console::note(&ci.summary_line());
    if report.has_gap() {
        console::blank();
        console::note(
            "Next: `signet ship --ci` (matrix), download artifacts, then `signet ship --collect DIR`.",
        );
        console::note(
            "Local `signet build` on this host will not fill off-host gaps by itself.",
        );
    } else {
        console::note(
            "Declared platforms have on-disk artifacts — verify signatures per OS before release.",
        );
    }
    if !ci.gaps.is_empty() {
        console::note("CI secrets gap — next: `signet ship secrets --push` (then `--apply`).");
    }
    Ok(())
}

fn emit_ci(ctx: &ProjectCtx, force: bool) -> anyhow::Result<()> {
    let rel = workflow_rel_path();
    let path = ctx.root.join(rel);
    if path.exists() && !force {
        anyhow::bail!(
            "{} already exists — pass --force to overwrite",
            path.display()
        );
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let body = render_signet_ship_workflow(&ctx.config);
    fs::write(&path, body)?;
    console::banner("ship · ci");
    console::ok_line(&format!("wrote {}", path.display()));
    let profile = assess_sign_profile(&ctx.config);
    console::note(&profile.summary_line());
    console::note("Commit the workflow, push a tag or run workflow_dispatch, then collect artifacts.");
    console::note("Push signing material with `signet ship secrets --push --apply` before expecting green CI.");
    if matches!(
        profile.path,
        crate::ship::ShipSignPath::Graduate
    ) {
        console::note(
            "Graduate path: also wire Azure/OV/Apple credentials in Actions before official Sign.",
        );
    }
    Ok(())
}

fn run_collect(ctx: &ProjectCtx, from: &std::path::Path) -> anyhow::Result<()> {
    console::banner("ship · collect");
    let report = collect_into_staging(&ctx.root, &ctx.config, from)?;
    console::ok_line(&format!(
        "copied {} file(s) → {} ({} staged total)",
        report.copied,
        report.staging.display(),
        report.staged_total
    ));
    if let Some(sums) = &report.sums_path {
        console::ok_line(&format!("wrote {}", sums.display()));
    }
    let cov = assess_coverage(&ctx.root, &ctx.config);
    console::blank();
    console::note(&cov.summary_line());
    if cov.has_gap() {
        console::note(&format!(
            "gap remains: {} — collect more OS artifacts or pass --allow-partial on release",
            cov.gap.join(", ")
        ));
    } else {
        console::note("coverage complete for declared platforms — ready for `signet release`");
    }
    Ok(())
}
