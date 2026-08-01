//! Tauri bundle filesystem discovery (low-level; used by [`crate::artifact::TauriAdapter`]).

use std::fs;
use std::path::{Path, PathBuf};

use crate::artifact::{Artifact, ArtifactKind};

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
pub fn discover_artifacts(src_tauri: &Path, profile: &str) -> anyhow::Result<Vec<Artifact>> {
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
            if let Some(kind) = ArtifactKind::classify_file(&path) {
                if !out.iter().any(|a| a.path == path) {
                    out.push(Artifact::new(path, kind));
                }
            }
        }
    }

    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn visit_dir(dir: &Path, out: &mut Vec<Artifact>) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // .app bundles are directories
            if path.extension().and_then(|e| e.to_str()) == Some("app") {
                out.push(Artifact::new(path, ArtifactKind::MacApp));
            } else {
                visit_dir(&path, out)?;
            }
        } else if let Some(kind) = ArtifactKind::classify_file(&path) {
            out.push(Artifact::new(path, kind));
        }
    }
    Ok(())
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
        assert_eq!(found[0].name_for_sums, "App_0.1.0_x64-setup.exe");
    }

    #[test]
    fn resolve_prefers_src_tauri_child() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src-tauri")).unwrap();
        let resolved = resolve_src_tauri(dir.path(), ".");
        assert!(resolved.ends_with("src-tauri"));
    }
}
