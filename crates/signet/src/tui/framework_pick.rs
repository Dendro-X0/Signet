//! Map scan project kinds → `project.framework` + build_command policy (Phase 14).

use crate::scan::ProjectKind;

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
];

/// Map a detected project kind to a config framework id.
pub fn framework_for_kind(kind: ProjectKind) -> Option<&'static str> {
    match kind {
        ProjectKind::Tauri => Some("tauri"),
        ProjectKind::Electron => Some("electron"),
        ProjectKind::Flutter => Some("flutter"),
        ProjectKind::ReactNative => Some("react-native"),
        ProjectKind::Expo => Some("expo"),
        ProjectKind::Capacitor => Some("capacitor"),
        ProjectKind::AndroidNative => Some("android"),
        ProjectKind::IosNative => Some("ios"),
    }
}

/// Prefer desktop/hybrid kinds when several are detected.
pub fn preferred_framework_from_kinds(kinds: &[ProjectKind]) -> Option<&'static str> {
    const ORDER: &[ProjectKind] = &[
        ProjectKind::Tauri,
        ProjectKind::Electron,
        ProjectKind::Flutter,
        ProjectKind::Expo,
        ProjectKind::ReactNative,
        ProjectKind::Capacitor,
        ProjectKind::AndroidNative,
        ProjectKind::IosNative,
    ];
    for want in ORDER {
        if kinds.contains(want) {
            return framework_for_kind(*want);
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
        _ => "optional for Tauri (uses tauri build)",
    }
}

pub fn index_of_framework(framework: &str) -> usize {
    let fw = framework.trim().to_ascii_lowercase();
    let fw = if fw == "rn" {
        "react-native"
    } else {
        fw.as_str()
    };
    FRAMEWORK_OPTIONS
        .iter()
        .position(|(id, _)| *id == fw)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_kinds() {
        assert_eq!(framework_for_kind(ProjectKind::Flutter), Some("flutter"));
        assert_eq!(framework_for_kind(ProjectKind::Expo), Some("expo"));
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
    fn build_command_required_for_hybrid() {
        assert!(requires_build_command("flutter"));
        assert!(requires_build_command("ios"));
        assert!(!requires_build_command("tauri"));
        assert!(!requires_build_command("electron"));
    }

    #[test]
    fn index_rn_alias() {
        assert_eq!(index_of_framework("rn"), index_of_framework("react-native"));
    }
}
