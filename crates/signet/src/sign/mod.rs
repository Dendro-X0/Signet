//! Platform signing + artifact helpers for `signet build`.

mod checksum;
mod discover;
mod linux;
mod macos;
mod pfx;
mod tools;
mod windows;

#[cfg(test)]
mod integration_tests;

use std::path::PathBuf;

use crate::identity::IdentityRecord;

pub use checksum::{verify_sha256sums, write_sha256sums, write_sha256sums_named, ChecksumResult};
pub use discover::{
    discover_artifacts, host_signable, resolve_src_tauri, ArtifactKind, DiscoveredArtifact,
};
pub use tools::{find_openssl, find_signtool, find_tauri_cli};

#[derive(Debug, Clone)]
pub struct SignOptions {
    /// Attempt Authenticode timestamp (Windows). Falls back if the server fails.
    pub timestamp: bool,
    pub timestamp_url: String,
}

impl Default for SignOptions {
    fn default() -> Self {
        Self {
            timestamp: true,
            timestamp_url: "http://timestamp.digicert.com".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SignedArtifact {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub kind: ArtifactKind,
    pub method: String,
    pub note: Option<String>,
}

#[derive(Debug, Default)]
pub struct SignReport {
    pub signed: Vec<SignedArtifact>,
    pub skipped: Vec<(PathBuf, String)>,
    pub warnings: Vec<String>,
}

/// Sign discovered artifacts for the current host OS.
pub fn sign_host_artifacts(
    identity: &IdentityRecord,
    artifacts: &[DiscoveredArtifact],
    opts: &SignOptions,
) -> anyhow::Result<SignReport> {
    match std::env::consts::OS {
        "windows" => windows::sign_artifacts(identity, artifacts, opts),
        "macos" => macos::sign_artifacts(identity, artifacts, opts),
        "linux" => linux::sign_artifacts(identity, artifacts, opts),
        other => anyhow::bail!("signing not supported on host OS '{other}'"),
    }
}

pub fn tool_hint_for_host() -> &'static str {
    match std::env::consts::OS {
        "windows" => "Windows SDK SignTool (signtool.exe) + OpenSSL for PFX export",
        "macos" => "codesign + security (Keychain)",
        "linux" => "openssl (detached signatures) — checksums always written",
        _ => "unsupported host",
    }
}
