//! Inspect report types.

use std::path::PathBuf;

use serde::Serialize;

use crate::artifact::ArtifactKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureStatus {
    Signed,
    Unsigned,
    Adhoc,
    Unknown,
    Error,
}

impl SignatureStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Signed => "signed",
            Self::Unsigned => "unsigned",
            Self::Adhoc => "adhoc",
            Self::Unknown => "unknown",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectRow {
    pub path: PathBuf,
    pub kind: String,
    pub platform: String,
    pub status: SignatureStatus,
    pub method: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectReport {
    pub schema_version: u32,
    pub files: Vec<InspectRow>,
    pub notes: Vec<String>,
}

impl InspectReport {
    pub fn new(files: Vec<InspectRow>) -> Self {
        Self {
            schema_version: 1,
            files,
            notes: vec![
                "Signed ≠ SmartScreen silence, Gatekeeper pass, Play trust, or notarization."
                    .into(),
                "Platform is the artifact's target ship platform, not the machine running inspect."
                    .into(),
            ],
        }
    }
}

pub fn platform_for_kind(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::WindowsExe | ArtifactKind::WindowsMsi => "windows",
        ArtifactKind::MacApp | ArtifactKind::MacDmg => "macos",
        ArtifactKind::LinuxAppImage | ArtifactKind::LinuxDeb | ArtifactKind::LinuxRpm => "linux",
        ArtifactKind::Apk | ArtifactKind::Aab => "android",
        ArtifactKind::Ipa => "ios",
        ArtifactKind::Zip | ArtifactKind::Other => "unknown",
    }
}
