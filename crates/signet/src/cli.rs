use clap::{Parser, Subcommand};

use crate::commands::{build, doctor, identity, init, release, scan, trust, verify};

/// Identity, sign, explain, and release self-signed desktop and mobile apps.
///
/// Run with no subcommand to open the interactive TUI hub.
#[derive(Debug, Parser)]
#[command(name = "signet", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create project config and local secrets layout
    Init(init::Args),
    /// Manage signing identity (create / import / list / show)
    Identity(identity::Args),
    /// Build and sign platform artifacts (Tauri today; more frameworks next)
    Build(build::Args),
    /// Emit trust / install documentation
    Trust(trust::Args),
    /// Checksums and publish release artifacts
    Release(release::Args),
    /// Check host tooling and prerequisites
    Doctor(doctor::Args),
    /// Scan the repo for installable apps and suggest signing config
    Scan(scan::Args),
    /// Verify fingerprints and SHA256SUMS for downloaded artifacts
    Verify(verify::Args),
}
