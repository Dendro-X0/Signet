//! Release keystore under `.signet/android/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::tools::find_keytool;

pub const ANDROID_DIR: &str = "android";
pub const KEYSTORE_FILE: &str = "release.jks";
pub const META_FILE: &str = "meta.toml";

#[derive(Debug, Clone)]
pub struct AndroidKeyPaths {
    pub dir: PathBuf,
    pub keystore: PathBuf,
    pub meta: PathBuf,
}

impl AndroidKeyPaths {
    pub fn from_secrets_dir(secrets_dir: &Path) -> Self {
        let dir = secrets_dir.join(ANDROID_DIR);
        Self {
            keystore: dir.join(KEYSTORE_FILE),
            meta: dir.join(META_FILE),
            dir,
        }
    }

    pub fn exists(&self) -> bool {
        self.keystore.is_file() && self.meta.is_file()
    }
}

pub fn keystore_paths(secrets_dir: &Path) -> AndroidKeyPaths {
    AndroidKeyPaths::from_secrets_dir(secrets_dir)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidKeyMeta {
    pub alias: String,
    pub store_type: String,
    pub created_at: String,
    /// SHA-256 fingerprint of the certificate (colon hex), when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cert_sha256: Option<String>,
    pub note: String,
}

pub fn store_pass() -> anyhow::Result<String> {
    std::env::var("SIGNET_ANDROID_STORE_PASS")
        .map_err(|_| {
            anyhow::anyhow!(
                "SIGNET_ANDROID_STORE_PASS is required (never store keystore passwords in signet.toml)"
            )
        })
        .map(|s| s.trim().to_string())
        .and_then(|s| {
            if s.is_empty() {
                Err(anyhow::anyhow!("SIGNET_ANDROID_STORE_PASS is empty"))
            } else {
                Ok(s)
            }
        })
}

pub fn key_pass() -> anyhow::Result<String> {
    if let Ok(p) = std::env::var("SIGNET_ANDROID_KEY_PASS") {
        let p = p.trim().to_string();
        if !p.is_empty() {
            return Ok(p);
        }
    }
    store_pass()
}

pub fn create_keystore(
    secrets_dir: &Path,
    alias: &str,
    dname: &str,
    force: bool,
) -> anyhow::Result<AndroidKeyPaths> {
    let paths = AndroidKeyPaths::from_secrets_dir(secrets_dir);
    if paths.exists() && !force {
        anyhow::bail!(
            "android keystore already exists at {} — pass --force to overwrite",
            paths.dir.display()
        );
    }
    let keytool = find_keytool().ok_or_else(|| {
        anyhow::anyhow!("keytool not found — install a JDK and ensure keytool is on PATH")
    })?;
    let pass = store_pass()?;
    let kpass = key_pass()?;

    fs::create_dir_all(&paths.dir)?;
    if force {
        let _ = fs::remove_file(&paths.keystore);
        let _ = fs::remove_file(&paths.meta);
    }

    let status = Command::new(&keytool)
        .args([
            "-genkeypair",
            "-v",
            "-keystore",
        ])
        .arg(&paths.keystore)
        .args(["-alias", alias, "-keyalg", "RSA", "-keysize", "2048"])
        .args(["-validity", "10000"])
        .args(["-storepass", &pass, "-keypass", &kpass])
        .args(["-dname", dname])
        .status()?;
    if !status.success() {
        anyhow::bail!("keytool -genkeypair failed with status {status}");
    }

    let cert_sha256 = read_cert_sha256(&paths.keystore, alias, &pass).ok();
    let meta = AndroidKeyMeta {
        alias: alias.into(),
        store_type: "JKS".into(),
        created_at: OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "unknown".into()),
        cert_sha256,
        note: "Local upload/sideload keystore — not the Play App Signing key.".into(),
    };
    fs::write(&paths.meta, toml::to_string_pretty(&meta)?)?;
    Ok(paths)
}

pub fn import_keystore(
    secrets_dir: &Path,
    source: &Path,
    alias: &str,
    force: bool,
) -> anyhow::Result<AndroidKeyPaths> {
    let paths = AndroidKeyPaths::from_secrets_dir(secrets_dir);
    if paths.exists() && !force {
        anyhow::bail!(
            "android keystore already exists at {} — pass --force to overwrite",
            paths.dir.display()
        );
    }
    if !source.is_file() {
        anyhow::bail!("keystore not found: {}", source.display());
    }
    fs::create_dir_all(&paths.dir)?;
    fs::copy(source, &paths.keystore)?;

    let pass = store_pass().ok();
    let cert_sha256 = pass
        .as_ref()
        .and_then(|p| read_cert_sha256(&paths.keystore, alias, p).ok());

    let meta = AndroidKeyMeta {
        alias: alias.into(),
        store_type: "JKS".into(),
        created_at: OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "unknown".into()),
        cert_sha256,
        note: "Imported keystore — verify alias; not the Play App Signing key.".into(),
    };
    fs::write(&paths.meta, toml::to_string_pretty(&meta)?)?;
    Ok(paths)
}

pub fn load_meta(paths: &AndroidKeyPaths) -> anyhow::Result<AndroidKeyMeta> {
    let text = fs::read_to_string(&paths.meta)?;
    Ok(toml::from_str(&text)?)
}

/// Parse SHA-256 certificate fingerprint from `keytool -list -v` output.
pub fn read_cert_sha256(keystore: &Path, alias: &str, store_pass: &str) -> anyhow::Result<String> {
    let keytool = find_keytool().ok_or_else(|| anyhow::anyhow!("keytool not found"))?;
    let out = Command::new(keytool)
        .args(["-list", "-v", "-keystore"])
        .arg(keystore)
        .args(["-alias", alias, "-storepass", store_pass])
        .output()?;
    if !out.status.success() {
        anyhow::bail!(
            "keytool -list failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let text = String::from_utf8_lossy(&out.stdout);
    parse_sha256_from_keytool(&text)
        .ok_or_else(|| anyhow::anyhow!("could not parse SHA-256 fingerprint from keytool output"))
}

pub fn parse_sha256_from_keytool(text: &str) -> Option<String> {
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if !(lower.contains("sha256:") || lower.contains("sha-256:")) {
            continue;
        }
        if let Some(idx) = line.find(':') {
            let rest = line[idx + 1..].trim();
            // keytool prints "SHA256: AA:BB:..." or "SHA256:\n\tAA:BB"
            let hex: String = rest
                .chars()
                .filter(|c| c.is_ascii_hexdigit() || *c == ':')
                .collect();
            let norm: String = hex.chars().filter(|c| c.is_ascii_hexdigit()).collect();
            if norm.len() == 64 {
                return Some(hex.trim().trim_matches(':').to_string());
            }
        }
    }
    // Multiline: "SHA256:" then next line with hex
    let lines: Vec<_> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("sha256") || lower.contains("sha-256") {
            if let Some(next) = lines.get(i + 1) {
                let hex: String = next
                    .chars()
                    .filter(|c| c.is_ascii_hexdigit() || *c == ':')
                    .collect();
                let norm: String = hex.chars().filter(|c| c.is_ascii_hexdigit()).collect();
                if norm.len() == 64 {
                    return Some(hex.trim().to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn paths_under_secrets() {
        let dir = tempdir().unwrap();
        let p = keystore_paths(dir.path());
        assert!(p.keystore.ends_with("android/release.jks") || p.keystore.ends_with("android\\release.jks"));
    }

    #[test]
    fn parse_keytool_sha256() {
        let sample = "SHA256: AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99\n";
        let parsed = parse_sha256_from_keytool(sample).expect("parse");
        let norm: String = parsed.chars().filter(|c| c.is_ascii_hexdigit()).collect();
        assert_eq!(norm.len(), 64);
    }

    #[test]
    fn meta_round_trip() {
        let dir = tempdir().unwrap();
        let paths = keystore_paths(dir.path());
        fs::create_dir_all(&paths.dir).unwrap();
        let meta = AndroidKeyMeta {
            alias: "signet".into(),
            store_type: "JKS".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            cert_sha256: Some("AA:BB".into()),
            note: "test".into(),
        };
        fs::write(&paths.meta, toml::to_string_pretty(&meta).unwrap()).unwrap();
        let loaded = load_meta(&paths).unwrap();
        assert_eq!(loaded.alias, "signet");
    }
}
