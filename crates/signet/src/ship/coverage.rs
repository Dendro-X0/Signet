//! Declared `[platforms]` vs on-disk artifacts (ship slice A).

use std::path::Path;

use crate::artifact::ArtifactKind;
use crate::config::{resolve_targets, Config};
use crate::scan::{scan_repository, Platform as ScanPlatform};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DesktopFlags {
    pub windows: bool,
    pub macos: bool,
    pub linux: bool,
}

impl DesktopFlags {
    pub fn from_config(config: &Config) -> Self {
        Self {
            windows: config.platforms.windows,
            macos: config.platforms.macos,
            linux: config.platforms.linux,
        }
    }

    pub fn merge_kind(&mut self, kind: ArtifactKind) {
        match kind {
            ArtifactKind::WindowsExe | ArtifactKind::WindowsMsi => self.windows = true,
            ArtifactKind::MacApp | ArtifactKind::MacDmg => self.macos = true,
            ArtifactKind::LinuxAppImage | ArtifactKind::LinuxDeb | ArtifactKind::LinuxRpm => {
                self.linux = true
            }
            _ => {}
        }
    }

    pub fn label_present(&self, name: &str) -> &'static str {
        let on = match name {
            "windows" => self.windows,
            "macos" => self.macos,
            "linux" => self.linux,
            _ => false,
        };
        if on {
            "present"
        } else {
            "MISSING"
        }
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
        format!(
            "ship coverage: windows={} macos={} linux={} (host={} can sign {} only)",
            self.declared_vs("windows"),
            self.declared_vs("macos"),
            self.declared_vs("linux"),
            self.host_os,
            self.host_can_sign,
        )
    }

    fn declared_vs(&self, name: &str) -> String {
        let declared = match name {
            "windows" => self.declared.windows,
            "macos" => self.declared.macos,
            "linux" => self.declared.linux,
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
        console::kv(14, "host", &format!("{} (can sign {})", self.host_os, self.host_can_sign));
        if self.gap.is_empty() {
            console::kv(14, "gap", "(none — declared desktop platforms have artifacts)");
        } else {
            console::kv(
                14,
                "gap",
                &format!(
                    "{} — need matching CI/host or `signet ship --collect` (planned)",
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
    format!(
        "windows={} macos={} linux={}",
        f.windows, f.macos, f.linux
    )
}

pub fn host_can_sign_platform() -> &'static str {
    match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "linux",
        other => other,
    }
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
                ScanPlatform::Android | ScanPlatform::Ios => {}
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

    let mut notes = Vec::new();
    notes.push(
        "[platforms] is a ship commitment: missing declared OS assets are a coverage gap, not optional docs."
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

    let mobile_targets: Vec<String> = resolve_targets(config)
        .into_iter()
        .filter(|t| {
            matches!(
                t.framework.as_str(),
                "expo" | "react-native" | "flutter" | "capacitor" | "android" | "ios"
            )
        })
        .map(|t| format!("{} ({})", t.id, t.framework))
        .collect();
    if !mobile_targets.is_empty() {
        notes.push(format!(
            "Mobile targets {} are not covered by [platforms] desktop flags yet — ship/mobile loop is later slice.",
            mobile_targets.join(", ")
        ));
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
    use crate::config::Config;
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
        CoverageReport {
            declared,
            present,
            host_os: host_os.into(),
            host_can_sign,
            gap,
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
            macos: false,
            linux: false,
        };
        let present = present_from_kinds(&[ArtifactKind::WindowsExe]);
        let report = coverage_from_parts(declared, present, "windows");
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
}
