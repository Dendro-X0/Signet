//! Minisign (+ optional GPG) attestation of `SHA256SUMS` (Phase 8).

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use minisign::{KeyPair, PublicKeyBox, SecretKeyBox, SignatureBox};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use which::which;

use crate::config::ChecksumSigning;

pub const SUMS_DIR_NAME: &str = "sums";
pub const MINISIGN_KEY_FILE: &str = "minisign.key";
pub const MINISIGN_PUB_FILE: &str = "minisign.pub";
pub const SUMS_META_FILE: &str = "meta.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SumsKeyMeta {
    pub created_at: String,
    pub algorithm: String,
    pub encrypted: bool,
}

#[derive(Debug, Clone)]
pub struct SumsKeyPaths {
    pub dir: PathBuf,
    pub secret: PathBuf,
    pub public: PathBuf,
    pub meta: PathBuf,
}

impl SumsKeyPaths {
    pub fn from_secrets_dir(secrets_dir: &Path) -> Self {
        let dir = secrets_dir.join(SUMS_DIR_NAME);
        Self {
            secret: dir.join(MINISIGN_KEY_FILE),
            public: dir.join(MINISIGN_PUB_FILE),
            meta: dir.join(SUMS_META_FILE),
            dir,
        }
    }

    pub fn exists(&self) -> bool {
        self.secret.is_file() && self.public.is_file()
    }
}

/// Create a Signet-managed minisign keypair under `.signet/sums/`.
///
/// Empty password by default (gitignored keys; CI-friendly). Set
/// `SIGNET_MINISIGN_PASSWORD` to encrypt the secret key.
pub fn create_sums_key(secrets_dir: &Path, force: bool) -> anyhow::Result<SumsKeyPaths> {
    let paths = SumsKeyPaths::from_secrets_dir(secrets_dir);
    if paths.exists() && !force {
        anyhow::bail!(
            "sums key already exists at {} — pass --force to overwrite",
            paths.dir.display()
        );
    }
    fs::create_dir_all(&paths.dir)?;

    let password = std::env::var("SIGNET_MINISIGN_PASSWORD").ok();
    let encrypted = password.as_ref().map(|p| !p.is_empty()).unwrap_or(false);
    let KeyPair { pk, sk } =
        KeyPair::generate_encrypted_keypair(Some(password.unwrap_or_default()))
            .map_err(|e| anyhow::anyhow!("minisign keygen failed: {e}"))?;

    let pk_box = pk
        .to_box()
        .map_err(|e| anyhow::anyhow!("minisign public box: {e}"))?;
    let sk_box = sk
        .to_box(Some("signet sums key"))
        .map_err(|e| anyhow::anyhow!("minisign secret box: {e}"))?;

    fs::write(&paths.public, pk_box.to_string())?;
    fs::write(&paths.secret, sk_box.to_string())?;

    let meta = SumsKeyMeta {
        created_at: OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "unknown".into()),
        algorithm: "minisign-ed25519".into(),
        encrypted,
    };
    fs::write(&paths.meta, toml::to_string_pretty(&meta)?)?;

    Ok(paths)
}

pub fn read_public_key_text(pub_path: &Path) -> anyhow::Result<String> {
    Ok(fs::read_to_string(pub_path)?)
}

fn load_secret_key(secret_path: &Path) -> anyhow::Result<minisign::SecretKey> {
    let text = fs::read_to_string(secret_path)?;
    let sk_box = SecretKeyBox::from_string(&text)
        .map_err(|e| anyhow::anyhow!("invalid minisign secret key: {e}"))?;
    let password = std::env::var("SIGNET_MINISIGN_PASSWORD").unwrap_or_default();
    sk_box
        .into_secret_key(Some(password))
        .map_err(|e| anyhow::anyhow!("failed to unlock minisign secret key: {e}"))
}

pub fn load_public_key_from_text(text: &str) -> anyhow::Result<minisign::PublicKey> {
    let pk_box = PublicKeyBox::from_string(text.trim())
        .map_err(|e| anyhow::anyhow!("invalid minisign public key: {e}"))?;
    pk_box
        .into_public_key()
        .map_err(|e| anyhow::anyhow!("invalid minisign public key: {e}"))
}

/// Sign `SHA256SUMS` bytes → sibling `SHA256SUMS.minisig`.
pub fn sign_sums_minisign(sums_path: &Path, secret_path: &Path) -> anyhow::Result<PathBuf> {
    let data = fs::read(sums_path)?;
    let sk = load_secret_key(secret_path)?;
    let comment = sums_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("SHA256SUMS");
    let trusted = format!(
        "timestamp:{}",
        OffsetDateTime::now_utc().unix_timestamp()
    );
    let signature_box = minisign::sign(None, &sk, Cursor::new(data), Some(comment), Some(&trusted))
        .map_err(|e| anyhow::anyhow!("minisign sign failed: {e}"))?;

    let out = sibling_path(sums_path, "minisig");
    fs::write(&out, signature_box.into_string())?;
    Ok(out)
}

/// Verify a minisign signature over the sums file.
pub fn verify_sums_minisign(
    sums_path: &Path,
    sig_path: &Path,
    public_key_text: &str,
) -> anyhow::Result<()> {
    let data = fs::read(sums_path)?;
    let sig_text = fs::read_to_string(sig_path)?;
    let signature_box = SignatureBox::from_string(&sig_text)
        .map_err(|e| anyhow::anyhow!("invalid minisign signature: {e}"))?;
    let pk = load_public_key_from_text(public_key_text)?;
    minisign::verify(&pk, &signature_box, Cursor::new(data), true, false, false)
        .map_err(|e| anyhow::anyhow!("minisign verify failed: {e}"))
}

/// Optional GPG armor detach-sign → `SHA256SUMS.asc`.
pub fn sign_sums_gpg(sums_path: &Path, key_id: &str) -> anyhow::Result<PathBuf> {
    let gpg = which("gpg").map_err(|_| anyhow::anyhow!("gpg not found on PATH"))?;
    let out = sibling_path(sums_path, "asc");
    if out.exists() {
        let _ = fs::remove_file(&out);
    }

    let mut cmd = Command::new(gpg);
    cmd.arg("--batch")
        .arg("--yes")
        .arg("--detach-sign")
        .arg("--armor")
        .arg("-o")
        .arg(&out)
        .arg(sums_path);

    if !key_id.trim().is_empty() {
        cmd.arg("--local-user").arg(key_id);
    }

    if let Ok(pass) = std::env::var("SIGNET_GPG_PASSPHRASE") {
        if !pass.is_empty() {
            cmd.arg("--pinentry-mode")
                .arg("loopback")
                .arg("--passphrase")
                .arg(pass);
        }
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("gpg detach-sign failed with status {status}");
    }
    if !out.is_file() {
        anyhow::bail!("gpg did not produce {}", out.display());
    }
    Ok(out)
}

pub fn verify_sums_gpg(sums_path: &Path, asc_path: &Path) -> anyhow::Result<()> {
    let gpg = which("gpg").map_err(|_| anyhow::anyhow!("gpg not found on PATH"))?;
    let status = Command::new(gpg)
        .arg("--batch")
        .arg("--verify")
        .arg(asc_path)
        .arg(sums_path)
        .status()?;
    if !status.success() {
        anyhow::bail!("gpg --verify failed with status {status}");
    }
    Ok(())
}

fn sibling_path(sums_path: &Path, ext: &str) -> PathBuf {
    let parent = sums_path.parent().unwrap_or_else(|| Path::new("."));
    let base = sums_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("SHA256SUMS");
    parent.join(format!("{base}.{ext}"))
}

#[derive(Debug, Clone, Default)]
pub struct SumsSignReport {
    pub minisig: Option<PathBuf>,
    pub asc: Option<PathBuf>,
    pub warnings: Vec<String>,
}

/// After writing `SHA256SUMS`, optionally sign with minisign/GPG per config.
pub fn maybe_sign_sums(
    sums_path: &Path,
    secrets_dir: &Path,
    cfg: &ChecksumSigning,
    no_sums_sign: bool,
    require_sums_sign: bool,
    require_gpg: bool,
) -> anyhow::Result<SumsSignReport> {
    let mut report = SumsSignReport::default();
    if no_sums_sign || !sums_path.is_file() {
        return Ok(report);
    }

    let paths = SumsKeyPaths::from_secrets_dir(secrets_dir);

    if cfg.minisign {
        if paths.exists() {
            match sign_sums_minisign(sums_path, &paths.secret) {
                Ok(p) => report.minisig = Some(p),
                Err(e) => {
                    if require_sums_sign {
                        return Err(e);
                    }
                    report.warnings.push(format!("minisign sign failed: {e}"));
                }
            }
        } else if require_sums_sign {
            anyhow::bail!(
                "checksum signing required but no minisign key — run `signet sums-key create`"
            );
        } else {
            report.warnings.push(
                "minisign enabled but no key at `.signet/sums/` — run `signet sums-key create`"
                    .into(),
            );
        }
    } else if require_sums_sign {
        anyhow::bail!("--require-sums-sign set but [trust.checksum_signing].minisign = false");
    }

    if cfg.gpg {
        match sign_sums_gpg(sums_path, &cfg.gpg_key_id) {
            Ok(p) => report.asc = Some(p),
            Err(e) => {
                if require_gpg {
                    return Err(e);
                }
                report
                    .warnings
                    .push(format!("gpg checksum signing skipped/failed: {e}"));
            }
        }
    } else if require_gpg {
        anyhow::bail!("--require-gpg set but [trust.checksum_signing].gpg = false");
    }

    Ok(report)
}

/// Extract minisign public key box text from TRUST.md when present.
pub fn parse_minisign_pub_from_trust(trust_md: &str) -> Option<String> {
    let mut in_minisign_section = false;
    let mut collecting = false;
    let mut buf = String::new();

    for line in trust_md.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let lower = trimmed.to_ascii_lowercase();
            in_minisign_section = lower.contains("minisign") || lower.contains("checksum signing");
            if collecting {
                break;
            }
            continue;
        }
        if !in_minisign_section && !collecting {
            continue;
        }
        if trimmed.starts_with("```") {
            if collecting {
                let body = buf.trim().to_string();
                if body.contains("untrusted comment:")
                    || body.lines().any(|l| l.trim().starts_with("RW"))
                {
                    return Some(body);
                }
                collecting = false;
                buf.clear();
            } else {
                collecting = true;
            }
            continue;
        }
        if collecting {
            buf.push_str(line);
            buf.push('\n');
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn create_sign_verify_round_trip() {
        let dir = tempdir().unwrap();
        let secrets = dir.path().join(".signet");
        let paths = create_sums_key(&secrets, false).unwrap();
        assert!(paths.public.is_file());
        assert!(paths.secret.is_file());

        let sums = dir.path().join("SHA256SUMS");
        fs::write(&sums, "abcdef0123456789  artifact.bin\n").unwrap();

        let sig = sign_sums_minisign(&sums, &paths.secret).unwrap();
        assert!(sig.is_file());

        let pub_text = read_public_key_text(&paths.public).unwrap();
        verify_sums_minisign(&sums, &sig, &pub_text).unwrap();

        fs::write(&sums, "deadbeef0123456789  artifact.bin\n").unwrap();
        assert!(verify_sums_minisign(&sums, &sig, &pub_text).is_err());
    }

    #[test]
    fn parse_minisign_from_trust_fence() {
        let trust = r#"# Trust

## Checksum signing (minisign)

Public key:

```
untrusted comment: minisign public key ABC
RWQabcdefghijklmnopqrstuvwxyz0123456789ABCD=
```

## Other
"#;
        let parsed = parse_minisign_pub_from_trust(trust).expect("pub");
        assert!(parsed.contains("RWQ"));
    }

    #[test]
    fn force_overwrite_key() {
        let dir = tempdir().unwrap();
        let secrets = dir.path().join(".signet");
        create_sums_key(&secrets, false).unwrap();
        assert!(create_sums_key(&secrets, false).is_err());
        create_sums_key(&secrets, true).unwrap();
    }
}
