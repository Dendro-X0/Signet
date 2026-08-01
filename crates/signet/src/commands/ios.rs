//! `signet ios` — IPA packaging + honesty notes (Phase 12).

use std::path::PathBuf;

use clap::{Args as ClapArgs, Subcommand};

use crate::ios::{default_ipa_path, honesty_notes, package_ipa, PackageResult};
use crate::project::ProjectCtx;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Path to signet.toml
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub action: Action,
}

#[derive(Debug, Subcommand)]
pub enum Action {
    /// Package an existing .app bundle into a .ipa (Payload/ zip)
    Package(PackageArgs),
    /// Print iOS trust / provisioning honesty notes
    Notes,
}

#[derive(Debug, ClapArgs)]
pub struct PackageArgs {
    /// Path to App.app bundle
    #[arg(long)]
    pub app: PathBuf,
    /// Output .ipa path (default: beside the .app)
    #[arg(long)]
    pub out: Option<PathBuf>,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    // Config optional for notes; required soft for package cwd context.
    let _ctx = ProjectCtx::load(args.config.as_deref()).ok();

    match args.action {
        Action::Notes => {
            println!("{}", honesty_notes());
            println!("Full guide: docs/ios.md");
        }
        Action::Package(p) => {
            let out = p.out.unwrap_or_else(|| default_ipa_path(&p.app));
            let result: PackageResult = package_ipa(&p.app, &out)?;
            println!("wrote {}", result.ipa_path.display());
            println!("bundled {}", result.app_name);
            println!("note: {}", honesty_notes());
        }
    }
    Ok(())
}
