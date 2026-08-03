//! Declared `[platforms]` (+ implied mobile targets) vs on-disk artifacts (ship slices A/G).

use std::path::Path;

use crate::artifact::ArtifactKind;
use crate::config::{resolve_targets, Config};
use crate::scan::{scan_repository, Platform as ScanPlatform};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DesktopFlags {
    pub windows: bool,
    pub macos: bool,
    pub linux: bool,
    pub android: bool,
    pub ios: bool,
}

impl DesktopFlags {
    pub fn from_config(config: &Config) -> Self {
        let (android, ios) = mobile_commitment(config);
        Self {
            windows: config.platforms.windows,
            macos: config.platforms.macos,
            linux: config.platforms.linux,
            android,
            ios,
        }
    }

    pub fn merge_kind(&mut self, kind: ArtifactKind) {
        match kind {
            ArtifactKind::WindowsExe | ArtifactKind::WindowsMsi => self.windows = true,
            ArtifactKind::MacApp | ArtifactKind::MacDmg => self.macos = true,
            ArtifactKind::LinuxAppImage | ArtifactKind::LinuxDeb | ArtifactKind::LinuxRpm => {
                self.linux = true
            }
            ArtifactKind::Apk | ArtifactKind::Aab => self.android = true,
            ArtifactKind::Ipa => self.ios = true,
            _ => {}
        }
    }

    pub fn label_present(&self, name: &str) -> &'static str {
        let on = match name {
            "windows" => self.windows,
            "macos" => self.macos,
            "linux" => self.linux,
            "android" => self.android,
            "ios" => self.ios,
            _ => false,
        };
        if on {
            "present"
        } else {
            "MISSING"
        }
    }
}

/// Android/iOS declared via `[platforms]` or mobile-style framework / `[[targets]]`.
pub fn mobile_commitment(config: &Config) -> (bool, bool) {
    let mut android = config.platforms.android;
    let mut ios = config.platforms.ios;
    imply_mobile_framework(config.project.framework.as_str(), &mut android, &mut ios);
    for t in resolve_targets(config) {
        imply_mobile_framework(t.framework.as_str(), &mut android, &mut ios);
    }
    (android, ios)
}

fn imply_mobile_framework(fw: &str, android: &mut bool, ios: &mut bool) {
    match fw.trim().to_ascii_lowercase().as_str() {
        "android" => *android = true,
        "ios" => *ios = true,
        "expo" | "react-native" | "rn" | "flutter" | "capacitor" => {
            *android = true;
            *ios = true;
        }
        _ => {}
    }
}

#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub declared: DesktopFlags,
    pub present: DesktopFlags,
    pub host_os: String,
    /// Desktop platform this host can host-sign today.
    pub host_can_sign: &'static str,
    pub gap: Vec<&'static str>,
    pub notes: Vec<String>,
}

impl CoverageReport {
    pub fn has_gap(&self) -> bool {
        !self.gap.is_empty()
    }

    /// One-line summary for build / doctor / guided.
    pub fn summary_line(&self) -> String {
        let mut parts = vec![
            format!("windows={}", self.declared_vs("windows")),
            format!("macos={}", self.declared_vs("macos")),
            format!("linux={}", self.declared_vs("linux")),
        ];
        if self.declared.android || self.declared.ios {
            parts.push(format!("android={}", self.declared_vs("android")));
            parts.push(format!("ios={}", self.declared_vs("ios")));
        }
        format!(
            "ship coverage: {} (host={} can sign {} only)",
            parts.join(" "),
            self.host_os,
            self.host_can_sign,
        )
    }

    fn declared_vs(&self, name: &str) -> String {
        let declared = match name {
            "windows" => self.declared.windows,
            "macos" => self.declared.macos,
            "linux" => self.declared.linux,
            "android" => self.declared.android,
            "ios" => self.declared.ios,
            _ => false,
        };
        if !declared {
            return "not-declared".into();
        }
        self.present.label_present(name).into()
    }

    pub fn print_human(&self) {
        use crate::ui::console;
        console::section("ship coverage");
        console::kv(14, "declared", &format_flags(&self.declared));
        console::kv(14, "present", &format_flags(&self.present));
        console::kv(
            14,
            "host",
            &format!("{} (can sign {})", self.host_os, self.host_can_sign),
        );
        if self.gap.is_empty() {
            console::kv(
                14,
                "gap",
                "(none — declared platforms have artifacts)",
            );
        } else {
            console::kv(
                14,
                "gap",
                &format!(
                    "{} — need matching CI/host or `signet ship --collect`",
                    self.gap.join(", ")
                ),
            );
        }
        for n in &self.notes {
            console::note(n);
        }
    }
}

fn format_flags(f: &DesktopFlags) -> String {
    let mut s = format!(
        "windows={} macos={} linux={}",
        f.windows, f.macos, f.linux
    );
    if f.android || f.ios {
        s.push_str(&format!(" android={} ios={}", f.android, f.ios));
    }
    s
}

pub fn host_can_sign_platform() -> &'static str {
    match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "linux",
        other => other,
    }
}

fn compute_gap(declared: &DesktopFlags, present: &DesktopFlags) -> Vec<&'static str> {
    let mut gap = Vec::new();
    if declared.windows && !present.windows {
        gap.push("windows");
    }
    if declared.macos && !present.macos {
        gap.push("macos");
    }
    if declared.linux && !present.linux {
        gap.push("linux");
    }
    if declared.android && !present.android {
        gap.push("android");
    }
    if declared.ios && !present.ios {
        gap.push("ios");
    }
    gap
}

/// Assess coverage from config + on-disk evidence (scan installers, discover, SHA256SUMS).
pub fn assess_coverage(root: &Path, config: &Config) -> CoverageReport {
    let declared = DesktopFlags::from_config(config);
    let mut present = DesktopFlags::default();

    // Prefer scan installer list (fast, framework-aware walk).
    if let Ok(scan) = scan_repository(root) {
        for inst in &scan.installers {
            match inst.platform {
                ScanPlatform::Windows => present.windows = true,
                ScanPlatform::Macos => present.macos = true,
                ScanPlatform::Linux => present.linux = true,
                ScanPlatform::Android => present.android = true,
                ScanPlatform::Ios => present.ios = true,
            }
        }
    }

    // Adapter discover (may find more under app_root).
    let ctx_ok = crate::project::ProjectCtx {
        config_path: root.join("signet.toml"),
        root: root.to_path_buf(),
        config: config.clone(),
    };
    for target in resolve_targets(config) {
        let tctx = ctx_ok.with_target(&target);
        if let Ok(adapter) = crate::artifact::select_adapter(&tctx.root, &tctx.config) {
            if let Ok(arts) = adapter.discover(&tctx, "release") {
                for a in arts {
                    present.merge_kind(a.kind);
                }
            }
        }
    }

    // Multi-host staging from `signet ship --collect`
    for path in crate::ship::staging_release_paths(root) {
        present.merge_kind(ArtifactKind::classify_explicit(&path));
    }

    // SHA256SUMS names (basename or relative).
    let sums = root.join("SHA256SUMS");
    if sums.is_file() {
        if let Ok(text) = std::fs::read_to_string(&sums) {
            if let Ok(entries) = crate::sign::parse_sha256sums(&text) {
                for (_, name) in entries {
                    let path = root.join(&name);
                    let kind = if path.is_file() {
                        ArtifactKind::classify_explicit(&path)
                    } else {
                        ArtifactKind::classify_explicit(Path::new(&name))
                    };
                    present.merge_kind(kind);
                }
            }
        }
    }

    let host_os = std::env::consts::OS.to_string();
    let host_can_sign = host_can_sign_platform();
    let gap = compute_gap(&declared, &present);

    let mut notes = Vec::new();
    notes.push(
        "[platforms] (+ mobile targets) is a ship commitment: missing declared OS assets are a coverage gap, not optional docs."
            .into(),
    );
    if declared.macos || declared.linux || declared.windows {
        let foreign: Vec<&str> = ["windows", "macos", "linux"]
            .into_iter()
            .filter(|p| {
                let declared_p = match *p {
                    "windows" => declared.windows,
                    "macos" => declared.macos,
                    "linux" => declared.linux,
                    _ => false,
                };
                declared_p && *p != host_can_sign
            })
            .collect();
        if !foreign.is_empty() {
            notes.push(format!(
                "This host will only produce/sign {host_can_sign}. Declared off-host platforms ({}) need CI or another machine — see `signet ship --plan`.",
                foreign.join(", ")
            ));
        }
    }

    if declared.android {
        notes.push(
            "Android: local keystore ≠ Play App Signing — see docs/android.md. Collect APK/AAB via CI or `signet ship --collect`."
                .into(),
        );
    }
    if declared.ios {
        notes.push(
            "iOS: free provisioning ~7 days; IPA packaging is not App Store trust — see docs/ios.md (macOS host / ship-ios job)."
                .into(),
        );
    }

    CoverageReport {
        declared,
        present,
        host_os,
        host_can_sign,
        gap,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Target};
    use tempfile::tempdir;

    fn present_from_kinds(kinds: &[ArtifactKind]) -> DesktopFlags {
        let mut f = DesktopFlags::default();
        for k in kinds {
            f.merge_kind(*k);
        }
        f
    }

    fn coverage_from_parts(
        declared: DesktopFlags,
        present: DesktopFlags,
        host_os: &str,
    ) -> CoverageReport {
        let host_can_sign = match host_os {
            "windows" => "windows",
            "macos" => "macos",
            "linux" => "linux",
            _ => "unknown",
        };
        CoverageReport {
            declared,
            present,
            host_os: host_os.into(),
            host_can_sign,
            gap: compute_gap(&declared, &present),
            notes: vec![
                "[platforms] is a ship commitment: missing declared OS assets are a coverage gap."
                    .into(),
            ],
        }
    }

    #[test]
    fn gap_when_only_windows_artifacts() {
        let declared = DesktopFlags {
            windows: true,
            macos: true,
            linux: true,
            ..Default::default()
        };
        let present = present_from_kinds(&[ArtifactKind::WindowsExe, ArtifactKind::WindowsMsi]);
        let report = coverage_from_parts(declared, present, "windows");
        assert!(report.has_gap());
        assert_eq!(report.gap, vec!["macos", "linux"]);
        assert!(report.summary_line().contains("macos=MISSING"));
        assert!(report.summary_line().contains("linux=MISSING"));
        assert!(report.summary_line().contains("windows=present"));
    }

    #[test]
    fn no_gap_when_undeclared() {
        let declared = DesktopFlags {
            windows: true,
            ..Default::default()
        };
        let present = present_from_kinds(&[ArtifactKind::WindowsExe]);
        let report = coverage_from_parts(declared, present, "windows");
        assert!(!report.has_gap());
    }

    #[test]
    fn expo_target_declares_android_and_ios() {
        let mut cfg = Config::example("App", ".");
        cfg.platforms.windows = true;
        cfg.platforms.macos = false;
        cfg.platforms.linux = false;
        cfg.targets.push(Target {
            id: "mobile".into(),
            framework: "expo".into(),
            app_root: "apps/mobile".into(),
            build_command: String::new(),
        });
        let (android, ios) = mobile_commitment(&cfg);
        assert!(android && ios);
        let declared = DesktopFlags::from_config(&cfg);
        assert!(declared.android && declared.ios);
        let report = coverage_from_parts(declared, DesktopFlags::default(), "windows");
        assert!(report.gap.contains(&"android"));
        assert!(report.gap.contains(&"ios"));
        assert!(report.summary_line().contains("android=MISSING"));
    }

    #[test]
    fn apk_fills_android_gap() {
        let declared = DesktopFlags {
            android: true,
            ..Default::default()
        };
        let present = present_from_kinds(&[ArtifactKind::Apk]);
        let report = coverage_from_parts(declared, present, "linux");
        assert!(!report.has_gap());
    }

    #[test]
    fn assess_finds_nested_windows_installer() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let nested = root.join("apps/desk/src-tauri/target/release/bundle/nsis");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("App_0.1.0_x64-setup.exe"), b"MZ").unwrap();
        let mut cfg = Config::example("App", "apps/desk");
        cfg.platforms.windows = true;
        cfg.platforms.macos = true;
        cfg.platforms.linux = true;
        cfg.write(root.join("signet.toml")).unwrap();
        let report = assess_coverage(root, &cfg);
        assert!(report.present.windows, "present={:?}", report.present);
        assert!(report.gap.contains(&"macos"));
        assert!(report.gap.contains(&"linux"));
    }

    #[test]
    fn assess_finds_apk_for_android_commitment() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let apk_dir = root.join("apps/mobile/android/app/build/outputs/apk/release");
        std::fs::create_dir_all(&apk_dir).unwrap();
        std::fs::write(apk_dir.join("app-release.apk"), b"APK").unwrap();
        let mut cfg = Config::example("App", "apps/desk");
        cfg.platforms.windows = false;
        cfg.platforms.macos = false;
        cfg.platforms.linux = false;
        cfg.platforms.android = true;
        cfg.targets.push(Target {
            id: "mobile".into(),
            framework: "expo".into(),
            app_root: "apps/mobile".into(),
            build_command: String::new(),
        });
        cfg.write(root.join("signet.toml")).unwrap();
        let report = assess_coverage(root, &cfg);
        assert!(report.present.android, "present={:?}", report.present);
        assert!(report.gap.contains(&"ios"), "gap={:?}", report.gap);
        assert!(!report.gap.contains(&"android"));
    }
}
