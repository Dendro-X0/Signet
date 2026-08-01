//! Map scan project kinds → `project.framework` + build_command policy (Phase 14).

use std::path::Path;

use crate::scan::{framework_id_for_kind, preferred_project, DetectedProject};

/// Framework ids offered in guided pick (stable order).
pub const FRAMEWORK_OPTIONS: &[(&str, &str)] = &[
    ("tauri", "Tauri"),
    ("electron", "Electron"),
    ("flutter", "Flutter"),
    ("react-native", "React Native"),
    ("expo", "Expo"),
    ("capacitor", "Capacitor"),
    ("android", "Android (APK)"),
    ("ios", "iOS"),
    ("cli", "Rust CLI / binary"),
];

/// Prefer shallow / non-demo detections (same ranking as `signet scan`).
pub fn preferred_framework_from_projects(
    root: &Path,
    projects: &[DetectedProject],
) -> Option<&'static str> {
    preferred_project(root, projects).map(|p| framework_id_for_kind(p.kind))
}

/// Prefer desktop/hybrid kinds when several are detected (kinds-only; no path ranking).
#[cfg(test)]
fn preferred_framework_from_kinds(kinds: &[crate::scan::ProjectKind]) -> Option<&'static str> {
    use crate::scan::ProjectKind;
    const ORDER: &[ProjectKind] = &[
        ProjectKind::Tauri,
        ProjectKind::Electron,
        ProjectKind::Flutter,
        ProjectKind::Expo,
        ProjectKind::ReactNative,
        ProjectKind::Capacitor,
        ProjectKind::AndroidNative,
        ProjectKind::IosNative,
        ProjectKind::RustCli,
    ];
    for want in ORDER {
        if kinds.contains(want) {
            return Some(framework_id_for_kind(*want));
        }
    }
    None
}

/// Adapters that refuse empty `build_command` on build (no safe default target).
pub fn requires_build_command(framework: &str) -> bool {
    matches!(
        framework.trim().to_ascii_lowercase().as_str(),
        "flutter" | "react-native" | "rn" | "expo" | "capacitor" | "ios"
    )
}

pub fn build_command_hint(framework: &str) -> &'static str {
    match framework.trim().to_ascii_lowercase().as_str() {
        "flutter" => "flutter build apk   (or: flutter build windows / macos / linux)",
        "react-native" | "rn" => "your RN release script (e.g. npm run build:android)",
        "expo" => "npx eas-cli build --local   (or leave empty and use --skip-build)",
        "capacitor" => "npm run build && npx cap sync   (then native pack)",
        "ios" => "xcodebuild -scheme App -configuration Release …",
        "electron" => "optional — empty defaults to: npm run dist",
        "android" => "optional — e.g. gradlew.bat assembleRelease",
        "cli" | "rust" | "rust-cli" => "optional — empty defaults to: cargo build --release",
        _ => "optional for Tauri (uses tauri build)",
    }
}

pub fn index_of_framework(framework: &str) -> usize {
    let fw = framework.trim().to_ascii_lowercase();
    let fw = match fw.as_str() {
        "rn" => "react-native",
        "rust" | "rust-cli" => "cli",
        other => other,
    };
    FRAMEWORK_OPTIONS
        .iter()
        .position(|(id, _)| *id == fw)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::{DetectedProject, ProjectKind};
    use std::path::PathBuf;

    #[test]
    fn maps_kinds() {
        assert_eq!(framework_id_for_kind(ProjectKind::Flutter), "flutter");
        assert_eq!(framework_id_for_kind(ProjectKind::Expo), "expo");
        assert_eq!(framework_id_for_kind(ProjectKind::RustCli), "cli");
    }

    #[test]
    fn prefers_tauri_over_android() {
        let kinds = [ProjectKind::AndroidNative, ProjectKind::Tauri];
        assert_eq!(preferred_framework_from_kinds(&kinds), Some("tauri"));
    }

    #[test]
    fn prefers_expo_over_rn() {
        let kinds = [ProjectKind::ReactNative, ProjectKind::Expo];
        assert_eq!(preferred_framework_from_kinds(&kinds), Some("expo"));
    }

    #[test]
    fn prefers_cli_over_demo_electron_by_path() {
        let root = PathBuf::from("/repo");
        let projects = vec![
            DetectedProject {
                kind: ProjectKind::RustCli,
                path: root.clone(),
                name: Some("Signet".into()),
                detail: "workspace".into(),
            },
            DetectedProject {
                kind: ProjectKind::Electron,
                path: root.join("demo/fixture"),
                name: Some("fixture".into()),
                detail: "electron".into(),
            },
        ];
        assert_eq!(
            preferred_framework_from_projects(&root, &projects),
            Some("cli")
        );
    }

    #[test]
    fn build_command_required_for_hybrid() {
        assert!(requires_build_command("flutter"));
        assert!(requires_build_command("ios"));
        assert!(!requires_build_command("tauri"));
        assert!(!requires_build_command("electron"));
        assert!(!requires_build_command("cli"));
    }

    #[test]
    fn index_rn_alias() {
        assert_eq!(index_of_framework("rn"), index_of_framework("react-native"));
        assert_eq!(index_of_framework("rust-cli"), index_of_framework("cli"));
    }
}
