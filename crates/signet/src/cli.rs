use clap::{Parser, Subcommand};

use crate::commands::{
    android, build, doctor, graduate, identity, init, ios, release, scan, self_cmd, sums_key, trust,
    verify,
};

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
    /// Manage minisign key for signing SHA256SUMS
    #[command(name = "sums-key")]
    SumsKey(sums_key::Args),
    /// Android keystore + APK signing helpers
    Android(android::Args),
    /// iOS IPA packaging helpers (no App Store trust claims)
    Ios(ios::Args),
    /// OV / Azure Trusted Signing / Apple notarization helpers
    Graduate(graduate::Args),
    /// Build and sign platform artifacts (Tauri / Electron / Android / iOS)
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
    /// Manage this Signet CLI install (update / uninstall)
    #[command(name = "self")]
    SelfCmd(self_cmd::Args),
}
