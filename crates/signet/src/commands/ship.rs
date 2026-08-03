//! `signet ship` — multi-platform coverage / plan (collect + CI later).

use std::path::PathBuf;

use clap::Args as ClapArgs;

use crate::project::ProjectCtx;
use crate::ship::assess_coverage;
use crate::ui::console;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to signet.toml
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Print declared vs present platform coverage (currently the only ship action)
    #[arg(long, default_value_t = true)]
    pub plan: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let ctx = ProjectCtx::load(args.config.as_deref()).map_err(|e| {
        anyhow::anyhow!("{e}\nhint: run `signet init` in your app directory first")
    })?;

    let _ = args.plan;
    let report = assess_coverage(&ctx.root, &ctx.config);

    console::banner("ship · plan");
    report.print_human();
    console::blank();
    console::note(&report.summary_line());
    if report.has_gap() {
        console::blank();
        console::note(
            "Next: run Signet on each missing OS (or upcoming `signet ship --ci`), then collect into one release.",
        );
        console::note(
            "Local `signet build` on this host will not fill the gap — that is expected until orchestration ships.",
        );
    } else {
        console::note(
            "Declared desktop platforms have on-disk artifacts — still verify signatures per OS before release.",
        );
    }
    Ok(())
}
