//! Signet — CLI + TUI for self-signed desktop and mobile distribution.

mod cli;
mod commands;
mod config;
mod error;
mod identity;
mod project;
mod release;
mod scan;
mod self_manage;
mod sign;
mod trust_kit;
mod trust_tier;
mod tui;
mod ui;

use clap::Parser;
use cli::{Cli, Command};
use error::ExitCode;

fn main() {
    let code = run().unwrap_or_else(|err| {
        eprintln!("error: {err}");
        if let Some(source) = err.source() {
            eprintln!("cause: {source}");
        }
        ExitCode::Failure
    });
    std::process::exit(code as i32);
}

fn run() -> anyhow::Result<ExitCode> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            tui::run_hub()?;
            Ok(ExitCode::Success)
        }
        Some(Command::Init(args)) => {
            commands::init::run(args)?;
            Ok(ExitCode::Success)
        }
        Some(Command::Identity(args)) => {
            commands::identity::run(args)?;
            Ok(ExitCode::Success)
        }
        Some(Command::Build(args)) => {
            commands::build::run(args)?;
            Ok(ExitCode::Success)
        }
        Some(Command::Trust(args)) => {
            commands::trust::run(args)?;
            Ok(ExitCode::Success)
        }
        Some(Command::Release(args)) => {
            commands::release::run(args)?;
            Ok(ExitCode::Success)
        }
        Some(Command::Doctor(args)) => {
            commands::doctor::run(args)?;
            Ok(ExitCode::Success)
        }
        Some(Command::Scan(args)) => {
            commands::scan::run(args)?;
            Ok(ExitCode::Success)
        }
        Some(Command::Verify(args)) => Ok(commands::verify::run(args)),
        Some(Command::SelfCmd(args)) => {
            commands::self_cmd::run(args)?;
            Ok(ExitCode::Success)
        }
    }
}
