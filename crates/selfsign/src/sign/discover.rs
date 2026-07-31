use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    WindowsExe,
    WindowsMsi,
    MacApp,
    MacDmg,
    LinuxAppImage,
    LinuxDeb,
    LinuxRpm,
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
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredArtifact {
    pub path: PathBuf,
    pub kind: ArtifactKind,
}

/// Resolve the `src-tauri` directory from project config.
pub fn resolve_src_tauri(project_root: &Path, tauri_root_rel: &str) -> PathBuf {
    let base = project_root.join(tauri_root_rel);
    if base.join("src-tauri").is_dir() {
        return base.join("src-tauri");
    }
    // Already pointing at src-tauri (or a crate with tauri.conf.json)
    if base.join("tauri.conf.json").is_file() || base.join("Cargo.toml").is_file() {
        return base;
    }
    base
}

/// Discover Tauri bundle outputs under `src-tauri/target/{profile}/bundle` plus main binary.
pub fn discover_artifacts(src_tauri: &Path, profile: &str) -> anyhow::Result<Vec<DiscoveredArtifact>> {
    let mut out = Vec::new();
    let target = src_tauri.join("target").join(profile);
    let bundle = target.join("bundle");

    if bundle.is_dir() {
        visit_dir(&bundle, &mut out)?;
    }

    // Main executable (unsigned until we sign it)
    if let Ok(entries) = fs::read_dir(&target) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(kind) = classify_file(&path) {
                if !out.iter().any(|a| a.path == path) {
                    out.push(DiscoveredArtifact { path, kind });
                }
            }
        }
    }

    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn visit_dir(dir: &Path, out: &mut Vec<DiscoveredArtifact>) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // .app bundles are directories
            if path.extension().and_then(|e| e.to_str()) == Some("app") {
                out.push(DiscoveredArtifact {
                    path,
                    kind: ArtifactKind::MacApp,
                });
            } else {
                visit_dir(&path, out)?;
            }
        } else if let Some(kind) = classify_file(&path) {
            out.push(DiscoveredArtifact { path, kind });
        }
    }
    Ok(())
}

fn classify_file(path: &Path) -> Option<ArtifactKind> {
    let name = path.file_name()?.to_str()?.to_ascii_lowercase();
    if name.ends_with(".appimage") {
        return Some(ArtifactKind::LinuxAppImage);
    }
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "exe" => Some(ArtifactKind::WindowsExe),
        "msi" | "msix" => Some(ArtifactKind::WindowsMsi),
        "dmg" => Some(ArtifactKind::MacDmg),
        "deb" => Some(ArtifactKind::LinuxDeb),
        "rpm" => Some(ArtifactKind::LinuxRpm),
        _ => None,
    }
}

/// Filter artifacts relevant to the current host (plus always keep checksums of all found).
pub fn host_signable(artifacts: &[DiscoveredArtifact]) -> Vec<DiscoveredArtifact> {
    let os = std::env::consts::OS;
    artifacts
        .iter()
        .filter(|a| match os {
            "windows" => matches!(a.kind, ArtifactKind::WindowsExe | ArtifactKind::WindowsMsi),
            "macos" => matches!(a.kind, ArtifactKind::MacApp | ArtifactKind::MacDmg),
            "linux" => matches!(
                a.kind,
                ArtifactKind::LinuxAppImage | ArtifactKind::LinuxDeb | ArtifactKind::LinuxRpm
            ),
            _ => false,
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn discovers_exe_under_bundle() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src-tauri");
        let bundle = src.join("target/release/bundle/nsis");
        fs::create_dir_all(&bundle).unwrap();
        let exe = bundle.join("App_0.1.0_x64-setup.exe");
        fs::write(&exe, b"MZ").unwrap();
        let found = discover_artifacts(&src, "release").unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, ArtifactKind::WindowsExe);
    }

    #[test]
    fn resolve_prefers_src_tauri_child() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src-tauri")).unwrap();
        let resolved = resolve_src_tauri(dir.path(), ".");
        assert!(resolved.ends_with("src-tauri"));
    }
}
