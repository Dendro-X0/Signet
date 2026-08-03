//! Self vs graduate Sign path on the same ship plan (slice F).

use std::env;
use std::path::{Path, PathBuf};

use crate::artifact::ArtifactKind;
use crate::config::{resolve_targets, Config};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShipSignPath {
    SelfSigned,
    Graduate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformSignAction {
    SelfSigned,
    Azure,
    Ov,
    Notarize,
    /// Linux (and similar): checksums / self integrity, no CA graduate helper.
    IntegrityOnly,
    GraduateMissing(String),
}

impl PlatformSignAction {
    pub fn label(&self) -> String {
        match self {
            Self::SelfSigned => "self".into(),
            Self::Azure => "graduate:azure".into(),
            Self::Ov => "graduate:ov".into(),
            Self::Notarize => "graduate:notarize".into(),
            Self::IntegrityOnly => "integrity (sums)".into(),
            Self::GraduateMissing(msg) => format!("graduate:MISSING ({msg})"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SignProfileReport {
    pub path: ShipSignPath,
    pub windows: PlatformSignAction,
    pub macos: PlatformSignAction,
    pub linux: PlatformSignAction,
    pub notes: Vec<String>,
}

impl SignProfileReport {
    pub fn summary_line(&self) -> String {
        let path = match self.path {
            ShipSignPath::SelfSigned => "self",
            ShipSignPath::Graduate => "graduate",
        };
        format!(
            "ship path={path}: windows={} macos={} linux={}",
            self.windows.label(),
            self.macos.label(),
            self.linux.label(),
        )
    }

    pub fn print_human(&self) {
        use crate::ui::console;
        console::section("ship sign path");
        console::kv(
            14,
            "path",
            match self.path {
                ShipSignPath::SelfSigned => "self",
                ShipSignPath::Graduate => "graduate",
            },
        );
        console::kv(14, "windows", &self.windows.label());
        console::kv(14, "macos", &self.macos.label());
        console::kv(14, "linux", &self.linux.label());
        for n in &self.notes {
            console::note(n);
        }
    }
}

pub fn parse_ship_path(raw: &str) -> ShipSignPath {
    match raw.trim().to_ascii_lowercase().as_str() {
        "graduate" | "official" => ShipSignPath::Graduate,
        _ => ShipSignPath::SelfSigned,
    }
}

pub fn assess_sign_profile(config: &Config) -> SignProfileReport {
    let path = parse_ship_path(&config.ship.path);
    let mut notes = Vec::new();

    let (windows, macos, linux) = match path {
        ShipSignPath::SelfSigned => {
            notes.push(
                "Default self-signed path — set [ship] path = \"graduate\" to use OV/Azure/notarize on the same plan."
                    .into(),
            );
            (
                PlatformSignAction::SelfSigned,
                PlatformSignAction::SelfSigned,
                PlatformSignAction::SelfSigned,
            )
        }
        ShipSignPath::Graduate => {
            notes.push(
                "Graduate path: helpers never fall back to Signet self-signed identity — see docs/graduation.md."
                    .into(),
            );
            notes.push(
                "CI: restore Azure/OV/Apple credentials via Actions secrets; never commit them."
                    .into(),
            );
            (
                resolve_windows_graduate(config),
                resolve_macos_graduate(config),
                PlatformSignAction::IntegrityOnly,
            )
        }
    };

    SignProfileReport {
        path,
        windows,
        macos,
        linux,
        notes,
    }
}

fn resolve_windows_graduate(config: &Config) -> PlatformSignAction {
    if azure_configured(config) {
        return PlatformSignAction::Azure;
    }
    if ov_configured(config) {
        return PlatformSignAction::Ov;
    }
    PlatformSignAction::GraduateMissing(
        "set [graduation.azure] dlib+metadata (or SIGNET_AZURE_*) or ov_thumbprint / SIGNET_OV_*"
            .into(),
    )
}

fn resolve_macos_graduate(config: &Config) -> PlatformSignAction {
    if apple_profile_configured(config) {
        PlatformSignAction::Notarize
    } else {
        PlatformSignAction::GraduateMissing(
            "set [graduation.apple] keychain_profile or SIGNET_NOTARY_PROFILE".into(),
        )
    }
}

pub fn azure_configured(config: &Config) -> bool {
    let dlib = env::var_os("SIGNET_AZURE_DLIB")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
        || !config.graduation.azure.dlib.trim().is_empty();
    let meta = env::var_os("SIGNET_AZURE_METADATA")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
        || !config.graduation.azure.metadata.trim().is_empty();
    dlib && meta
}

pub fn ov_configured(config: &Config) -> bool {
    !config.graduation.ov_thumbprint.trim().is_empty()
        || env::var_os("SIGNET_OV_THUMBPRINT")
            .map(|v| !v.is_empty())
            .unwrap_or(false)
        || env::var_os("SIGNET_OV_PFX")
            .map(|v| !v.is_empty())
            .unwrap_or(false)
}

pub fn apple_profile_configured(config: &Config) -> bool {
    !config.graduation.apple.keychain_profile.trim().is_empty()
        || env::var_os("SIGNET_NOTARY_PROFILE")
            .map(|v| !v.is_empty())
            .unwrap_or(false)
}

/// Discover host installers suitable for graduate apply on this OS.
pub fn discover_graduate_files(root: &Path, config: &Config) -> Vec<PathBuf> {
    let os = std::env::consts::OS;
    let mut out = Vec::new();
    let ctx = crate::project::ProjectCtx {
        config_path: root.join("signet.toml"),
        root: root.to_path_buf(),
        config: config.clone(),
    };
    for target in resolve_targets(config) {
        let tctx = ctx.with_target(&target);
        let Ok(adapter) = crate::artifact::select_adapter(&tctx.root, &tctx.config) else {
            continue;
        };
        let Ok(arts) = adapter.discover(&tctx, "release") else {
            continue;
        };
        for a in arts {
            if graduate_kind_for_os(a.kind, os) {
                out.push(a.path);
            }
        }
    }
    for path in crate::ship::staging_release_paths(root) {
        let kind = ArtifactKind::classify_explicit(&path);
        if graduate_kind_for_os(kind, os) {
            out.push(path);
        }
    }
    out.sort();
    out.dedup();
    out
}

fn graduate_kind_for_os(kind: ArtifactKind, os: &str) -> bool {
    match os {
        "windows" => matches!(kind, ArtifactKind::WindowsExe | ArtifactKind::WindowsMsi),
        "macos" => matches!(kind, ArtifactKind::MacApp | ArtifactKind::MacDmg),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn default_path_is_self() {
        let cfg = Config::example("app", ".");
        let report = assess_sign_profile(&cfg);
        assert_eq!(report.path, ShipSignPath::SelfSigned);
        assert_eq!(report.windows, PlatformSignAction::SelfSigned);
        assert!(report.summary_line().contains("path=self"));
    }

    #[test]
    fn graduate_prefers_azure_over_ov() {
        let mut cfg = Config::example("app", ".");
        cfg.ship.path = "graduate".into();
        cfg.graduation.azure.dlib = "dlib.dll".into();
        cfg.graduation.azure.metadata = "meta.json".into();
        cfg.graduation.ov_thumbprint = "ABCDEF".into();
        let report = assess_sign_profile(&cfg);
        assert_eq!(report.path, ShipSignPath::Graduate);
        assert_eq!(report.windows, PlatformSignAction::Azure);
        assert!(matches!(
            report.macos,
            PlatformSignAction::GraduateMissing(_)
        ));
        assert_eq!(report.linux, PlatformSignAction::IntegrityOnly);
    }

    #[test]
    fn graduate_ov_when_no_azure() {
        let mut cfg = Config::example("app", ".");
        cfg.ship.path = "graduate".into();
        cfg.graduation.ov_thumbprint = "ABCDEF".into();
        let report = assess_sign_profile(&cfg);
        assert_eq!(report.windows, PlatformSignAction::Ov);
    }

    #[test]
    fn graduate_macos_when_profile_set() {
        let mut cfg = Config::example("app", ".");
        cfg.ship.path = "graduate".into();
        cfg.graduation.apple.keychain_profile = "Notary".into();
        let report = assess_sign_profile(&cfg);
        assert_eq!(report.macos, PlatformSignAction::Notarize);
    }

    #[test]
    fn ship_path_round_trips_in_toml() {
        let mut cfg = Config::example("app", ".");
        cfg.ship.path = "graduate".into();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("signet.toml");
        cfg.write(&path).unwrap();
        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.ship.path, "graduate");
        assert_eq!(parse_ship_path(&loaded.ship.path), ShipSignPath::Graduate);
    }

    #[test]
    fn discover_finds_staged_exe_on_windows_host() {
        if std::env::consts::OS != "windows" {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let staging = root.join("dist/signet-ship");
        std::fs::create_dir_all(&staging).unwrap();
        std::fs::write(staging.join("App-setup.exe"), b"MZ").unwrap();
        let cfg = Config::example("App", ".");
        let found = discover_graduate_files(root, &cfg);
        assert!(
            found.iter().any(|p| p.ends_with("App-setup.exe")),
            "found={found:?}"
        );
    }
}
