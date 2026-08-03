//! Merge multi-host installers into `dist/signet-ship/` + rewrite SHA256SUMS.

use std::fs;
use std::path::{Path, PathBuf};

use crate::artifact::ArtifactKind;
use crate::config::Config;
use crate::sign::{maybe_sign_sums, write_sha256sums};

pub const STAGING_DIR: &str = "dist/signet-ship";

/// Copy installer-like files from `from_dir` into `{root}/dist/signet-ship/`, rewrite sums.
pub fn collect_into_staging(
    root: &Path,
    config: &Config,
    from_dir: &Path,
) -> anyhow::Result<CollectReport> {
    if !from_dir.is_dir() {
        anyhow::bail!("collect source is not a directory: {}", from_dir.display());
    }

    let staging = root.join(STAGING_DIR);
    fs::create_dir_all(&staging)?;

    let mut copied = Vec::new();
    let mut visited = 0usize;
    walk_copy(from_dir, &staging, &mut copied, &mut visited, 0)?;

    // Include anything already staged.
    let mut all_files = list_staging_files(&staging)?;
    for p in &copied {
        if !all_files.iter().any(|x| x == p) {
            all_files.push(p.clone());
        }
    }

    let sums_path = root.join("SHA256SUMS");
    if !all_files.is_empty() {
        write_sha256sums(&sums_path, &all_files)?;
        let secrets = config.secrets_path(root);
        let sign_report = maybe_sign_sums(
            &sums_path,
            &secrets,
            &config.trust.checksum_signing,
            false,
            false,
            false,
        )?;
        for w in &sign_report.warnings {
            eprintln!("warning: {w}");
        }
    }

    Ok(CollectReport {
        staging,
        copied: copied.len(),
        staged_total: all_files.len(),
        sums_path: if sums_path.is_file() {
            Some(sums_path)
        } else {
            None
        },
    })
}

#[derive(Debug)]
pub struct CollectReport {
    pub staging: PathBuf,
    pub copied: usize,
    pub staged_total: usize,
    pub sums_path: Option<PathBuf>,
}

fn list_staging_files(staging: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !staging.is_dir() {
        return Ok(out);
    }
    for entry in fs::read_dir(staging)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && is_collectable(&path) {
            out.push(path);
        }
    }
    Ok(out)
}

fn walk_copy(
    dir: &Path,
    staging: &Path,
    copied: &mut Vec<PathBuf>,
    visited: &mut usize,
    depth: usize,
) -> anyhow::Result<()> {
    if depth > 16 || *visited > 80_000 {
        return Ok(());
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        *visited += 1;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if path.is_dir() {
            if matches!(
                name_str.as_ref(),
                "node_modules" | ".git" | ".signet" | ".next" | "debug" | "deps" | "incremental"
            ) {
                continue;
            }
            walk_copy(&path, staging, copied, visited, depth + 1)?;
        } else if path.is_file() && is_collectable(&path) {
            let dest = unique_dest(staging, &name_str)?;
            fs::copy(&path, &dest)?;
            copied.push(dest);
            let sib = path.with_file_name(format!("{name_str}.sig"));
            if sib.is_file() {
                let sig_dest = unique_dest(staging, &format!("{name_str}.sig"))?;
                fs::copy(&sib, &sig_dest)?;
                copied.push(sig_dest);
            }
        }
    }
    Ok(())
}

fn is_collectable(path: &Path) -> bool {
    let kind = ArtifactKind::classify_explicit(path);
    matches!(
        kind,
        ArtifactKind::WindowsExe
            | ArtifactKind::WindowsMsi
            | ArtifactKind::MacDmg
            | ArtifactKind::MacApp
            | ArtifactKind::LinuxAppImage
            | ArtifactKind::LinuxDeb
            | ArtifactKind::LinuxRpm
            | ArtifactKind::Apk
            | ArtifactKind::Aab
            | ArtifactKind::Ipa
    ) || path
        .file_name()
        .and_then(|s| s.to_str())
        .map(|n| {
            let lower = n.to_ascii_lowercase();
            lower == "sha256sums"
                || lower.ends_with(".minisig")
                || lower.ends_with(".sig")
                || lower == "trust.md"
        })
        .unwrap_or(false)
}

fn unique_dest(staging: &Path, name: &str) -> anyhow::Result<PathBuf> {
    let mut dest = staging.join(name);
    if !dest.exists() {
        return Ok(dest);
    }
    let stem = Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("asset");
    let ext = Path::new(name)
        .extension()
        .and_then(|s| s.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    for i in 2..1000 {
        dest = staging.join(format!("{stem}-{i}{ext}"));
        if !dest.exists() {
            return Ok(dest);
        }
    }
    anyhow::bail!("could not find unique name for {name} under {}", staging.display());
}

/// Paths under staging for release attach.
pub fn staging_release_paths(root: &Path) -> Vec<PathBuf> {
    let staging = root.join(STAGING_DIR);
    list_staging_files(&staging).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use tempfile::tempdir;

    #[test]
    fn collect_copies_exe_and_writes_sums() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("proj");
        fs::create_dir_all(&root).unwrap();
        let src = dir.path().join("artifacts/win");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("App_1.0.0_x64-setup.exe"), b"MZ").unwrap();

        let cfg = Config::example("App", ".");
        let report = collect_into_staging(&root, &cfg, &src).unwrap();
        assert_eq!(report.copied, 1);
        assert!(root.join(STAGING_DIR).join("App_1.0.0_x64-setup.exe").is_file());
        assert!(root.join("SHA256SUMS").is_file());
        let text = fs::read_to_string(root.join("SHA256SUMS")).unwrap();
        assert!(text.contains("dist/signet-ship/") || text.contains("App_1.0.0"));
    }
}
