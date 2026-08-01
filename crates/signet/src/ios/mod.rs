//! iOS IPA packaging helpers (Phase 12) — no App Store trust claims.

mod package;

pub use package::{default_ipa_path, package_ipa, PackageResult};

/// Short honesty text for CLI / TRUST.
pub fn honesty_notes() -> &'static str {
    "iOS free Apple ID development provisioning typically lasts ~7 days. \
     Packaging an IPA does not grant App Store, TestFlight, or notarization trust. \
     See docs/ios.md."
}
