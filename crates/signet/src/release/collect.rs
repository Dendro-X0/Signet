use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::artifact::select_adapter;
use crate::config::Config;
use crate::project::ProjectCtx;
use crate::sign::{maybe_sign_sums, write_sha256sums_named};

#[derive(Debug, Clone)]
pub struct ReleaseFile {
    pub path: PathBuf,
    /// Name used as the GitHub asset filename
    pub asset_name: String,
    pub kind: &'static str,
}

#[derive(Debug, Clone, Default)]
pub struct CollectOpts {
    pub no_sums_sign: bool,
    pub require_sums_sign: bool,
    pub require_gpg: bool,
    /// When true (release `--dry-run`), never rewrite or sign `SHA256SUMS` on disk.
    pub read_only: bool,
}

/// Gather uploadable files: discovered bundles, sidecar .sig, SHA256SUMS, optional TRUST.md.
pub fn collect_release_files_with_opts(
    project_root: &Path,
    config: &Config,
    profile: &str,
    extra_artifacts: &[PathBuf],
    attach_trust: bool,
    opts: CollectOpts,
) -> anyhow::Result<Vec<ReleaseFile>> {
    let mut paths: Vec<PathBuf> = Vec::new();

    if extra_artifacts.is_empty() {
        let ctx = ProjectCtx {
            config_path: project_root.join("signet.toml"),
            root: project_root.to_path_buf(),
            config: config.clone(),
        };
        for target in crate::config::resolve_targets(config) {
            let tctx = ctx.with_target(&target);
            let adapter = select_adapter(project_root, &tctx.config)?;
            for art in adapter.discover(&tctx, profile)? {
                if art.path.is_file() {
                    let sig = sibling_sig(&art.path);
                    paths.push(art.path);
                    if sig.is_file() {
                        paths.push(sig);
                    }
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
        if opts.read_only {
            if sums_path.is_file() {
                by_name.insert("SHA256SUMS".into(), sums_path.clone());
            }
            // Do not create/rewrite sums or signatures in dry-run.
        } else {
            write_sha256sums_named(&sums_path, &named)?;
            by_name.insert("SHA256SUMS".into(), sums_path.clone());

            let secrets = config.secrets_path(project_root);
            let sign_report = maybe_sign_sums(
                &sums_path,
                &secrets,
                &config.trust.checksum_signing,
                opts.no_sums_sign,
                opts.require_sums_sign,
                opts.require_gpg,
            )?;
            for w in &sign_report.warnings {
                eprintln!("warning: {w}");
            }
            if let Some(p) = sign_report.minisig {
                by_name.insert("SHA256SUMS.minisig".into(), p);
            }
            if let Some(p) = sign_report.asc {
                by_name.insert("SHA256SUMS.asc".into(), p);
            }
        }
    }

    // Attach pre-existing signature siblings if signing was skipped but files remain.
    for name in ["SHA256SUMS.minisig", "SHA256SUMS.asc"] {
        let p = project_root.join(name);
        if p.is_file() {
            by_name.entry(name.into()).or_insert(p);
        }
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
    } else if lower == "sha256sums.minisig" || lower == "sha256sums.asc" {
        "checksums-sig"
    } else if lower == "trust.md" {
        "trust"
    } else if lower.ends_with(".sig") || lower.ends_with(".minisig") || lower.ends_with(".asc") {
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

/// Ensure SHA256SUMS lists every file that will be uploaded (except itself, TRUST, and sum sigs).
/// When `lenient` (dry-run), missing sums or missing basename entries become warnings, not errors.
pub fn verify_checksums_cover_opts(files: &[ReleaseFile], lenient: bool) -> anyhow::Result<()> {
    let Some(sums) = files.iter().find(|f| f.asset_name == "SHA256SUMS") else {
        if lenient {
            eprintln!(
                "warning: no SHA256SUMS on disk — dry-run will not create one; live `signet release` rewrites flat basenames"
            );
            return Ok(());
        }
        return Ok(());
    };
    let text = fs::read_to_string(&sums.path)?;
    for f in files {
        if skip_checksum_coverage(&f.asset_name) {
            continue;
        }
        if !text.contains(&f.asset_name) {
            let msg = format!(
                "SHA256SUMS missing entry for {} — re-run collect or `signet build`",
                f.asset_name
            );
            if lenient {
                eprintln!(
                    "warning: {msg} (dry-run keeps existing sums; live release rewrites to asset basenames)"
                );
            } else {
                anyhow::bail!("{msg}");
            }
        }
    }
    Ok(())
}

fn skip_checksum_coverage(asset_name: &str) -> bool {
    matches!(
        asset_name,
        "SHA256SUMS" | "SHA256SUMS.minisig" | "SHA256SUMS.asc" | "TRUST.md"
    )
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
        let files =
            collect_release_files_with_opts(root, &cfg, "release", &[], true, CollectOpts::default())
                .unwrap();
        let names: Vec<_> = files.iter().map(|f| f.asset_name.as_str()).collect();
        assert!(names.contains(&"app-setup.exe"));
        assert!(names.contains(&"SHA256SUMS"));
        assert!(names.contains(&"TRUST.md"));
        verify_checksums_cover_opts(&files, false).unwrap();
    }

    #[test]
    fn read_only_does_not_rewrite_or_create_sums() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let bundle = root.join("src-tauri/target/release/bundle/nsis");
        fs::create_dir_all(&bundle).unwrap();
        fs::write(bundle.join("app-setup.exe"), b"MZ").unwrap();

        let sums = root.join("SHA256SUMS");
        let original = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  apps/nested/app-setup.exe\n";
        fs::write(&sums, original).unwrap();

        let cfg = Config::example("app", ".");
        let opts = CollectOpts {
            read_only: true,
            ..CollectOpts::default()
        };
        let files =
            collect_release_files_with_opts(root, &cfg, "release", &[], false, opts).unwrap();
        assert!(files.iter().any(|f| f.asset_name == "app-setup.exe"));
        assert_eq!(fs::read_to_string(&sums).unwrap(), original);

        // No sums file case: must not create one.
        fs::remove_file(&sums).unwrap();
        let files2 = collect_release_files_with_opts(
            root,
            &cfg,
            "release",
            &[],
            false,
            CollectOpts {
                read_only: true,
                ..CollectOpts::default()
            },
        )
        .unwrap();
        assert!(!sums.is_file(), "dry-run must not create SHA256SUMS");
        assert!(!files2.iter().any(|f| f.asset_name == "SHA256SUMS"));
        verify_checksums_cover_opts(&files2, true).unwrap();
    }
}
