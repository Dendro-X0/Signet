//! `signet ship` — coverage plan, CI workflow emit, multi-host collect.

use std::fs;
use std::path::PathBuf;

use clap::Args as ClapArgs;

use crate::project::ProjectCtx;
use crate::ship::{
    assess_coverage, collect_into_staging, render_signet_ship_workflow, workflow_rel_path,
};
use crate::ui::console;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to signet.toml
    #[arg(long)]
    pub config: Option<PathBuf>,

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

pub fn run(args: Args) -> anyhow::Result<()> {
    let ctx = ProjectCtx::load(args.config.as_deref()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `signet init` in your app directory first")
    })?;

    if args.ci {
        return emit_ci(&ctx, args.force);
    }
    if let Some(dir) = args.collect {
        return run_collect(&ctx, &dir);
    }

    let _ = args.plan;
    let report = assess_coverage(&ctx.root, &ctx.config);

    console::banner("ship · plan");
    report.print_human();
    console::blank();
    console::note(&report.summary_line());
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
            "Declared desktop platforms have on-disk artifacts — verify signatures per OS before release.",
        );
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
    console::note("Commit the workflow, push a tag or run workflow_dispatch, then collect artifacts.");
    console::note("Restore `.signet/identity` in CI via secrets before expecting host signatures.");
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
        console::note("coverage complete for declared desktop platforms — ready for `signet release`");
    }
    Ok(())
}
