//! Trust tier ids — integrity labels (not OS reputation).

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config::Config;

/// Primary integrity tier. Snake_case ids match docs/trust-model.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustTier {
    ChecksumOnly,
    SelfSignedHost,
    CommunitySignedSums,
    CaAuthenticode,
    AppleNotarized,
    PlayManaged,
    Unknown,
}

impl TrustTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ChecksumOnly => "checksum_only",
            Self::SelfSignedHost => "self_signed_host",
            Self::CommunitySignedSums => "community_signed_sums",
            Self::CaAuthenticode => "ca_authenticode",
            Self::AppleNotarized => "apple_notarized",
            Self::PlayManaged => "play_managed",
            Self::Unknown => "unknown",
        }
    }

    pub fn meaning(self) -> &'static str {
        match self {
            Self::ChecksumOnly => {
                "Artifacts are covered by SHA256SUMS; no host crypto signature is asserted."
            }
            Self::SelfSignedHost => {
                "Artifacts are (or will be) host-signed with a self-issued Signet identity."
            }
            Self::CommunitySignedSums => {
                "SHA256SUMS is attested with a community signature (minisign/GPG)."
            }
            Self::CaAuthenticode => {
                "Windows signature is declared to chain to a public CA (graduation)."
            }
            Self::AppleNotarized => {
                "macOS builds are declared Developer ID signed and notarized (graduation)."
            }
            Self::PlayManaged => {
                "Android distribution uses Play App Signing (external to local keystore)."
            }
            Self::Unknown => "Not enough evidence to name an integrity tier.",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "checksum_only" => Some(Self::ChecksumOnly),
            "self_signed_host" => Some(Self::SelfSignedHost),
            "community_signed_sums" => Some(Self::CommunitySignedSums),
            "ca_authenticode" => Some(Self::CaAuthenticode),
            "apple_notarized" => Some(Self::AppleNotarized),
            "play_managed" => Some(Self::PlayManaged),
            "unknown" => Some(Self::Unknown),
            "" => None,
            _ => None,
        }
    }
}

impl std::fmt::Display for TrustTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Inputs for inferring a primary tier when `[trust].declared_tier` is unset.
#[derive(Debug, Clone, Copy)]
pub struct TierHints {
    pub has_active_identity: bool,
    pub has_sha256sums: bool,
    pub has_sums_signature: bool,
}

impl TierHints {
    pub fn probe(project_root: &Path) -> Self {
        let has_active_identity = project_root.join(".signet/identity/active").is_file()
            || project_root.join(".selfsign/identity/active").is_file();
        let has_sha256sums = project_root.join("SHA256SUMS").is_file();
        let has_sums_signature = project_root.join("SHA256SUMS.minisig").is_file()
            || project_root.join("SHA256SUMS.asc").is_file();
        Self {
            has_active_identity,
            has_sha256sums,
            has_sums_signature,
        }
    }
}

/// Resolve primary tier: declared wins when valid; else infer from hints.
pub fn resolve_primary_tier(config: &Config, hints: TierHints) -> TrustTier {
    if let Some(raw) = config
        .trust
        .declared_tier
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if let Some(t) = TrustTier::parse(raw) {
            return t;
        }
    }

    if hints.has_sums_signature {
        return TrustTier::CommunitySignedSums;
    }
    if hints.has_active_identity {
        return TrustTier::SelfSignedHost;
    }
    if hints.has_sha256sums {
        return TrustTier::ChecksumOnly;
    }
    TrustTier::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn declared_overrides_inference() {
        let mut cfg = Config::default();
        cfg.trust.declared_tier = Some("checksum_only".into());
        let tier = resolve_primary_tier(
            &cfg,
            TierHints {
                has_active_identity: true,
                has_sha256sums: true,
                has_sums_signature: false,
            },
        );
        assert_eq!(tier, TrustTier::ChecksumOnly);
    }

    #[test]
    fn infers_self_signed_with_identity() {
        let cfg = Config::default();
        let tier = resolve_primary_tier(
            &cfg,
            TierHints {
                has_active_identity: true,
                has_sha256sums: false,
                has_sums_signature: false,
            },
        );
        assert_eq!(tier, TrustTier::SelfSignedHost);
    }
}
