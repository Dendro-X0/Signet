//! `signet inspect` — best-effort signed / unsigned / platform report.

use std::path::PathBuf;

use clap::Args as ClapArgs;

use crate::error::ExitCode;
use crate::inspect::{inspect_path, InspectReport, SignatureStatus};
use crate::ui::console;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Artifact path(s) to inspect
    #[arg(long = "file", required = true)]
    pub files: Vec<PathBuf>,
    /// JSON report for agents
    #[arg(long)]
    pub json: bool,
    /// Exit 1 if any file is unsigned or error
    #[arg(long)]
    pub strict: bool,
}

pub fn run(args: Args) -> ExitCode {
    match run_inner(args) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::NotImplemented
        }
    }
}

fn run_inner(args: Args) -> anyhow::Result<ExitCode> {
    let mut rows = Vec::new();
    for f in &args.files {
        rows.push(inspect_path(f));
    }
    let report = InspectReport::new(rows);

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human(&report);
    }

    if args.strict
        && report
            .files
            .iter()
            .any(|r| matches!(r.status, SignatureStatus::Unsigned | SignatureStatus::Error))
    {
        return Ok(ExitCode::Failure);
    }
    Ok(ExitCode::Success)
}

fn print_human(report: &InspectReport) {
    console::section("Signature inspect");
    for row in &report.files {
        console::kv(12, "path", &row.path.display().to_string());
        console::kv(12, "platform", &row.platform);
        console::kv(12, "kind", &row.kind);
        console::kv(12, "status", row.status.as_str());
        console::kv(12, "method", &row.method);
        if !row.detail.is_empty() {
            console::kv(12, "detail", &row.detail);
        }
        println!();
    }
    for note in &report.notes {
        println!("note: {note}");
    }
}
