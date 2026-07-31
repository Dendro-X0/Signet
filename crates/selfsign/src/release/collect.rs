use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::sign::{discover_artifacts, resolve_src_tauri, write_sha256sums_named};

#[derive(Debug, Clone)]
pub struct ReleaseFile {
    pub path: PathBuf,
    /// Name used as the GitHub asset filename
    pub asset_name: String,
    pub kind: &'static str,
}

/// Gather uploadable files: discovered bundles, sidecar .sig, SHA256SUMS, optional TRUST.md.
pub fn collect_release_files(
    project_root: &Path,
    config: &Config,
    profile: &str,
    extra_artifacts: &[PathBuf],
    attach_trust: bool,
) -> anyhow::Result<Vec<ReleaseFile>> {
    let src_tauri = resolve_src_tauri(project_root, &config.project.tauri_root);
    let mut paths: Vec<PathBuf> = Vec::new();

    if extra_artifacts.is_empty() {
        for art in discover_artifacts(&src_tauri, profile)? {
            if art.path.is_file() {
                let sig = sibling_sig(&art.path);
                paths.push(art.path);
                if sig.is_file() {
                    paths.push(sig);
                }
            }
        }
    } else {
        for p in extra_artifacts {
            if p.is_file() {
                paths.push(p.clone());
                let sig = sibling_sig(p);
                if sig.is_file() {
                    paths.push(sig);
                }
            } else if p.is_dir() {
                anyhow::bail!(
                    "cannot upload directory as release asset: {} (use a .dmg/.zip instead)",
                    p.display()
                );
            } else {
                anyhow::bail!("artifact not found: {}", p.display());
            }
        }
    }

    let mut by_name: BTreeMap<String, PathBuf> = BTreeMap::new();
    for path in paths {
        let name = unique_asset_name(&path, &by_name)?;
        by_name.insert(name, path);
    }

    let named: Vec<(String, PathBuf)> = by_name
        .iter()
        .map(|(n, p)| (n.clone(), p.clone()))
        .collect();
    let sums_path = project_root.join("SHA256SUMS");
    if !named.is_empty() {
        write_sha256sums_named(&sums_path, &named)?;
        by_name.insert("SHA256SUMS".into(), sums_path);
    }

    if attach_trust {
        let trust = project_root.join("TRUST.md");
        if trust.is_file() {
            by_name.insert("TRUST.md".into(), trust);
        }
    }

    let mut out = Vec::new();
    for (asset_name, path) in by_name {
        let kind = classify_kind(&asset_name);
        out.push(ReleaseFile {
            path,
            asset_name,
            kind,
        });
    }
    Ok(out)
}

fn sibling_sig(path: &Path) -> PathBuf {
    let name = format!(
        "{}.sig",
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("artifact")
    );
    path.with_file_name(name)
}

fn unique_asset_name(path: &Path, existing: &BTreeMap<String, PathBuf>) -> anyhow::Result<String> {
    let base = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid artifact name: {}", path.display()))?
        .to_string();
    if !existing.contains_key(&base) {
        return Ok(base);
    }
    let parent = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("asset");
    let alt = format!("{parent}-{base}");
    if existing.contains_key(&alt) {
        anyhow::bail!("duplicate asset name for {}", path.display());
    }
    Ok(alt)
}

fn classify_kind(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower == "sha256sums" {
        "checksums"
    } else if lower == "trust.md" {
        "trust"
    } else if lower.ends_with(".sig") {
        "signature"
    } else if lower.ends_with(".exe") || lower.ends_with(".msi") || lower.ends_with(".msix") {
        "windows"
    } else if lower.ends_with(".dmg") || lower.ends_with(".pkg") {
        "macos"
    } else if lower.ends_with(".appimage") || lower.ends_with(".deb") || lower.ends_with(".rpm") {
        "linux"
    } else {
        "other"
    }
}

/// Ensure SHA256SUMS lists every file that will be uploaded (except itself and TRUST.md).
pub fn verify_checksums_cover(files: &[ReleaseFile]) -> anyhow::Result<()> {
    let Some(sums) = files.iter().find(|f| f.asset_name == "SHA256SUMS") else {
        return Ok(());
    };
    let text = fs::read_to_string(&sums.path)?;
    for f in files {
        if f.asset_name == "SHA256SUMS" || f.asset_name == "TRUST.md" {
            continue;
        }
        if !text.contains(&f.asset_name) {
            anyhow::bail!(
                "SHA256SUMS missing entry for {} — re-run collect or `selfsign build`",
                f.asset_name
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use tempfile::tempdir;

    #[test]
    fn collects_exe_checksums_and_trust() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let bundle = root.join("src-tauri/target/release/bundle/nsis");
        fs::create_dir_all(&bundle).unwrap();
        fs::write(bundle.join("app-setup.exe"), b"MZ").unwrap();
        fs::write(root.join("TRUST.md"), "# trust\n").unwrap();

        let cfg = Config::example("app", ".");
        let files = collect_release_files(root, &cfg, "release", &[], true).unwrap();
        let names: Vec<_> = files.iter().map(|f| f.asset_name.as_str()).collect();
        assert!(names.contains(&"app-setup.exe"));
        assert!(names.contains(&"SHA256SUMS"));
        assert!(names.contains(&"TRUST.md"));
        verify_checksums_cover(&files).unwrap();
    }
}
