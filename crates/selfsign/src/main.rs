//! selfsign — CLI + TUI for self-signed Tauri distribution.

mod cli;
mod commands;
mod config;
mod error;
mod identity;
mod project;
mod release;
mod scan;
mod sign;
mod trust_kit;
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
        None => tui::run_hub()?,
        Some(Command::Init(args)) => commands::init::run(args)?,
        Some(Command::Identity(args)) => commands::identity::run(args)?,
        Some(Command::Build(args)) => commands::build::run(args)?,
        Some(Command::Trust(args)) => commands::trust::run(args)?,
        Some(Command::Release(args)) => commands::release::run(args)?,
        Some(Command::Doctor(args)) => commands::doctor::run(args)?,
        Some(Command::Scan(args)) => commands::scan::run(args)?,
    }

    Ok(ExitCode::Success)
}
