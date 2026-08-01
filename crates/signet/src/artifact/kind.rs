//! Artifact kind ids (stable for JSON / agents).

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    WindowsExe,
    WindowsMsi,
    MacApp,
    MacDmg,
    LinuxAppImage,
    LinuxDeb,
    LinuxRpm,
    /// Reserved — Phase 11
    Apk,
    /// Android App Bundle (Play upload; not auto-signed as Play distribution key)
    Aab,
    /// Reserved — Phase 12
    Ipa,
    Zip,
    Other,
}

impl ArtifactKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WindowsExe => "windows-exe",
            Self::WindowsMsi => "windows-msi",
            Self::MacApp => "macos-app",
            Self::MacDmg => "macos-dmg",
            Self::LinuxAppImage => "linux-appimage",
            Self::LinuxDeb => "linux-deb",
            Self::LinuxRpm => "linux-rpm",
            Self::Apk => "android-apk",
            Self::Aab => "android-aab",
            Self::Ipa => "ios-ipa",
            Self::Zip => "zip",
            Self::Other => "other",
        }
    }

    /// Classify by extension / name. Returns `None` when the file is not a known installable.
    pub fn classify_file(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_str()?.to_ascii_lowercase();
        if name.ends_with(".appimage") {
            return Some(Self::LinuxAppImage);
        }
        if name.ends_with(".apk") {
            return Some(Self::Apk);
        }
        if name.ends_with(".aab") {
            return Some(Self::Aab);
        }
        if name.ends_with(".ipa") {
            return Some(Self::Ipa);
        }
        match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
            "exe" => Some(Self::WindowsExe),
            "msi" | "msix" => Some(Self::WindowsMsi),
            "dmg" => Some(Self::MacDmg),
            "deb" => Some(Self::LinuxDeb),
            "rpm" => Some(Self::LinuxRpm),
            "zip" => Some(Self::Zip),
            _ => None,
        }
    }

    /// Like [`classify_file`], but always returns a kind (fallback `Other`).
    pub fn classify_explicit(path: &Path) -> Self {
        if path.extension().and_then(|e| e.to_str()) == Some("app")
            || path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|n| n.to_ascii_lowercase().ends_with(".app"))
                .unwrap_or(false)
        {
            return Self::MacApp;
        }
        Self::classify_file(path).unwrap_or(Self::Other)
    }

    pub fn host_signable_on(self, os: &str) -> bool {
        match os {
            "windows" => matches!(self, Self::WindowsExe | Self::WindowsMsi),
            "macos" => matches!(self, Self::MacApp | Self::MacDmg),
            "linux" => matches!(
                self,
                Self::LinuxAppImage | Self::LinuxDeb | Self::LinuxRpm
            ),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn kind_ids_stable() {
        assert_eq!(ArtifactKind::WindowsExe.as_str(), "windows-exe");
        assert_eq!(ArtifactKind::Apk.as_str(), "android-apk");
        assert_eq!(ArtifactKind::Ipa.as_str(), "ios-ipa");
    }

    #[test]
    fn classify_exe_and_apk() {
        assert_eq!(
            ArtifactKind::classify_file(Path::new("setup.exe")),
            Some(ArtifactKind::WindowsExe)
        );
        assert_eq!(
            ArtifactKind::classify_file(Path::new("app.apk")),
            Some(ArtifactKind::Apk)
        );
        assert_eq!(
            ArtifactKind::classify_explicit(&PathBuf::from("weird.bin")),
            ArtifactKind::Other
        );
    }
}
