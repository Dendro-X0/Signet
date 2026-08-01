//! Sign APKs with apksigner (preferred) or jarsigner.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::keystore::{key_pass, load_meta, store_pass, AndroidKeyPaths};
use super::tools::{find_apksigner, find_jarsigner};

#[derive(Debug, Default)]
pub struct AndroidSignReport {
    pub signed: Vec<(PathBuf, String)>,
    pub skipped: Vec<(PathBuf, String)>,
    pub warnings: Vec<String>,
}

pub fn sign_apks(paths: &AndroidKeyPaths, apks: &[PathBuf]) -> anyhow::Result<AndroidSignReport> {
    let mut report = AndroidSignReport::default();
    if !paths.keystore.is_file() {
        anyhow::bail!(
            "android keystore missing at {} — run `signet android keystore create`",
            paths.keystore.display()
        );
    }
    let meta = load_meta(paths)?;
    let store = store_pass()?;
    let key = key_pass()?;

    for apk in apks {
        if !apk.is_file() {
            report
                .skipped
                .push((apk.clone(), "file not found".into()));
            continue;
        }
        let name = apk
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if name.ends_with(".aab") {
            report.warnings.push(format!(
                "skipping {}: AAB Play upload signing is documented in docs/android.md — \
                 Signet does not treat a local keystore as the Play app signing key",
                apk.display()
            ));
            report
                .skipped
                .push((apk.clone(), "aab — see docs/android.md".into()));
            continue;
        }
        if !name.ends_with(".apk") {
            report
                .skipped
                .push((apk.clone(), "not an .apk".into()));
            continue;
        }
        match sign_apk(apk, paths, &meta.alias, &store, &key) {
            Ok(method) => report.signed.push((apk.clone(), method)),
            Err(e) => report.skipped.push((apk.clone(), e.to_string())),
        }
    }
    Ok(report)
}

pub fn sign_apk(
    apk: &Path,
    paths: &AndroidKeyPaths,
    alias: &str,
    store_pass: &str,
    key_pass: &str,
) -> anyhow::Result<String> {
    if let Some(apksigner) = find_apksigner() {
        let status = Command::new(&apksigner)
            .arg("sign")
            .arg("--ks")
            .arg(&paths.keystore)
            .args(["--ks-key-alias", alias])
            .arg("--ks-pass")
            .arg(format!("pass:{store_pass}"))
            .arg("--key-pass")
            .arg(format!("pass:{key_pass}"))
            .arg(apk)
            .status()?;
        if status.success() {
            return Ok(format!("apksigner ({})", apksigner.display()));
        }
        anyhow::bail!("apksigner sign failed with status {status}");
    }

    let jarsigner = find_jarsigner().ok_or_else(|| {
        anyhow::anyhow!(
            "neither apksigner nor jarsigner found — install Android SDK build-tools or a JDK"
        )
    })?;
    let status = Command::new(&jarsigner)
        .args(["-verbose", "-sigalg", "SHA256withRSA", "-digestalg", "SHA-256"])
        .args(["-keystore"])
        .arg(&paths.keystore)
        .args(["-storepass", store_pass, "-keypass", key_pass])
        .arg(apk)
        .arg(alias)
        .status()?;
    if !status.success() {
        anyhow::bail!("jarsigner failed with status {status}");
    }
    Ok(format!("jarsigner ({})", jarsigner.display()))
}
