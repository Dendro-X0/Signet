use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use rcgen::{
    CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose,
};
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

use super::fingerprint::{fingerprint_from_cert_pem, fingerprint_sha256_colon, pem_to_der};

const ACTIVE_FILE: &str = "active";
const META_FILE: &str = "meta.toml";
const CERT_FILE: &str = "cert.pem";
const KEY_FILE: &str = "key.pem";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityMeta {
    pub name: String,
    pub common_name: String,
    #[serde(default)]
    pub organization: String,
    pub fingerprint_sha256: String,
    pub created_at: String,
    pub not_before: String,
    pub not_after: String,
    pub key_algorithm: String,
}

#[derive(Debug, Clone)]
pub struct IdentityRecord {
    pub meta: IdentityMeta,
    pub dir: PathBuf,
    #[allow(dead_code)] // retained for Phase 3 signing / export
    pub cert_pem: String,
    /// Present when loaded from disk for signing later; never print this.
    pub key_pem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePointer {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct CreateOptions {
    pub name: String,
    pub common_name: String,
    pub organization: String,
    pub days: u32,
    pub force: bool,
}

#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub name: String,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub force: bool,
}

pub fn create_identity(identity_root: &Path, opts: &CreateOptions) -> anyhow::Result<IdentityRecord> {
    let dir = identity_root.join(&opts.name);
    if dir.exists() && !opts.force {
        anyhow::bail!(
            "identity '{}' already exists at {} (pass --force to overwrite)",
            opts.name,
            dir.display()
        );
    }

    let key_pair = KeyPair::generate()?;
    let mut params = CertificateParams::default();
    params.distinguished_name.push(DnType::CommonName, opts.common_name.clone());
    if !opts.organization.is_empty() {
        params
            .distinguished_name
            .push(DnType::OrganizationName, opts.organization.clone());
    }
    params.is_ca = IsCa::NoCa;
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::CodeSigning];

    let now = OffsetDateTime::now_utc();
    let not_after = now + Duration::days(i64::from(opts.days));
    params.not_before = now;
    params.not_after = not_after;

    // Leaf code-signing cert (not a CA)
    let cert = params.self_signed(&key_pair)?;
    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();
    let der = cert.der();
    let fingerprint = fingerprint_sha256_colon(der.as_ref());

    let meta = IdentityMeta {
        name: opts.name.clone(),
        common_name: opts.common_name.clone(),
        organization: opts.organization.clone(),
        fingerprint_sha256: fingerprint,
        created_at: now.format(&Rfc3339)?,
        not_before: now.format(&Rfc3339)?,
        not_after: not_after.format(&Rfc3339)?,
        key_algorithm: "ECDSA_P256".into(),
    };

    write_identity_dir(&dir, &meta, &cert_pem, &key_pem)?;
    set_active(identity_root, &opts.name)?;

    Ok(IdentityRecord {
        meta,
        dir,
        cert_pem,
        key_pem,
    })
}

pub fn import_identity(identity_root: &Path, opts: &ImportOptions) -> anyhow::Result<IdentityRecord> {
    let dir = identity_root.join(&opts.name);
    if dir.exists() && !opts.force {
        anyhow::bail!(
            "identity '{}' already exists at {} (pass --force to overwrite)",
            opts.name,
            dir.display()
        );
    }

    let cert_pem = fs::read_to_string(&opts.cert_path)?;
    let key_pem = fs::read_to_string(&opts.key_path)?;
    if !cert_pem.contains("BEGIN CERTIFICATE") {
        anyhow::bail!("cert file does not look like a PEM certificate");
    }
    if !key_pem.contains("BEGIN") || !key_pem.contains("PRIVATE KEY") {
        anyhow::bail!("key file does not look like a PEM private key");
    }

    let der = pem_to_der(&cert_pem)?;
    let (_, parsed) = x509_parser::parse_x509_certificate(&der)
        .map_err(|e| anyhow::anyhow!("failed to parse certificate: {e}"))?;

    let common_name = parsed
        .subject()
        .iter_common_name()
        .next()
        .and_then(|cn| cn.as_str().ok())
        .unwrap_or("imported")
        .to_string();

    let organization = parsed
        .subject()
        .iter_organization()
        .next()
        .and_then(|o| o.as_str().ok())
        .unwrap_or("")
        .to_string();

    let not_before = OffsetDateTime::from_unix_timestamp(parsed.validity().not_before.timestamp())
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .format(&Rfc3339)?;
    let not_after = OffsetDateTime::from_unix_timestamp(parsed.validity().not_after.timestamp())
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .format(&Rfc3339)?;

    let fingerprint = fingerprint_sha256_colon(&der);
    let now = OffsetDateTime::now_utc().format(&Rfc3339)?;

    let meta = IdentityMeta {
        name: opts.name.clone(),
        common_name,
        organization,
        fingerprint_sha256: fingerprint,
        created_at: now,
        not_before,
        not_after,
        key_algorithm: "imported".into(),
    };

    write_identity_dir(&dir, &meta, &cert_pem, &key_pem)?;
    set_active(identity_root, &opts.name)?;

    Ok(IdentityRecord {
        meta,
        dir,
        cert_pem,
        key_pem,
    })
}

pub fn list_identities(identity_root: &Path) -> anyhow::Result<Vec<IdentityMeta>> {
    if !identity_root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(identity_root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let meta_path = entry.path().join(META_FILE);
        if !meta_path.exists() {
            continue;
        }
        let meta: IdentityMeta = toml::from_str(&fs::read_to_string(meta_path)?)?;
        out.push(meta);
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn load_named(identity_root: &Path, name: &str) -> anyhow::Result<IdentityRecord> {
    let dir = identity_root.join(name);
    load_from_dir(&dir)
}

pub fn load_active(identity_root: &Path) -> anyhow::Result<IdentityRecord> {
    let active = read_active(identity_root)?;
    load_named(identity_root, &active.name)
}

pub fn set_active(identity_root: &Path, name: &str) -> anyhow::Result<()> {
    fs::create_dir_all(identity_root)?;
    let dir = identity_root.join(name);
    if !dir.join(META_FILE).exists() {
        anyhow::bail!("identity '{name}' not found under {}", identity_root.display());
    }
    let pointer = ActivePointer {
        name: name.to_string(),
    };
    let text = toml::to_string_pretty(&pointer)?;
    fs::write(identity_root.join(ACTIVE_FILE), text)?;
    Ok(())
}

pub fn read_active(identity_root: &Path) -> anyhow::Result<ActivePointer> {
    let path = identity_root.join(ACTIVE_FILE);
    if !path.exists() {
        anyhow::bail!("no active identity — run `signet identity create`");
    }
    let text = fs::read_to_string(path)?;
    // Support plain name file or TOML
    if text.trim().starts_with("name") || text.contains('[') {
        Ok(toml::from_str(&text)?)
    } else {
        Ok(ActivePointer {
            name: text.trim().to_string(),
        })
    }
}

fn load_from_dir(dir: &Path) -> anyhow::Result<IdentityRecord> {
    let meta_path = dir.join(META_FILE);
    let cert_path = dir.join(CERT_FILE);
    let key_path = dir.join(KEY_FILE);
    if !meta_path.exists() {
        anyhow::bail!("identity metadata missing at {}", meta_path.display());
    }
    let meta: IdentityMeta = toml::from_str(&fs::read_to_string(meta_path)?)?;
    let cert_pem = fs::read_to_string(cert_path)?;
    let key_pem = fs::read_to_string(key_path)?;

    // Refresh fingerprint from cert if meta is stale
    let fp = fingerprint_from_cert_pem(&cert_pem)?;
    let mut meta = meta;
    if meta.fingerprint_sha256 != fp {
        meta.fingerprint_sha256 = fp;
    }

    Ok(IdentityRecord {
        meta,
        dir: dir.to_path_buf(),
        cert_pem,
        key_pem,
    })
}

fn write_identity_dir(
    dir: &Path,
    meta: &IdentityMeta,
    cert_pem: &str,
    key_pem: &str,
) -> anyhow::Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    fs::create_dir_all(dir)?;

    let meta_text = toml::to_string_pretty(meta)?;
    fs::write(dir.join(META_FILE), meta_text)?;
    fs::write(dir.join(CERT_FILE), cert_pem)?;
    write_secret_file(&dir.join(KEY_FILE), key_pem)?;
    Ok(())
}

fn write_secret_file(path: &Path, contents: &str) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(contents.as_bytes())?;
    }
    #[cfg(not(unix))]
    {
        fs::write(path, contents)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn create_list_show_round_trip() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("identity");
        let rec = create_identity(
            &root,
            &CreateOptions {
                name: "default".into(),
                common_name: "Demo App".into(),
                organization: "Demo Org".into(),
                days: 365,
                force: false,
            },
        )
        .unwrap();

        assert!(!rec.meta.fingerprint_sha256.is_empty());
        assert!(root.join("default").join(KEY_FILE).exists());
        assert!(root.join(ACTIVE_FILE).exists());

        let listed = list_identities(&root).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "default");

        let active = load_active(&root).unwrap();
        assert_eq!(
            active.meta.fingerprint_sha256,
            rec.meta.fingerprint_sha256
        );
        assert!(active.key_pem.contains("PRIVATE KEY"));
    }

    #[test]
    fn import_pem_round_trip() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("identity");
        let created = create_identity(
            &root,
            &CreateOptions {
                name: "source".into(),
                common_name: "Import Me".into(),
                organization: String::new(),
                days: 30,
                force: false,
            },
        )
        .unwrap();

        let cert_path = created.dir.join(CERT_FILE);
        let key_path = created.dir.join(KEY_FILE);
        let imported = import_identity(
            &root,
            &ImportOptions {
                name: "copy".into(),
                cert_path,
                key_path,
                force: false,
            },
        )
        .unwrap();

        assert_eq!(
            imported.meta.fingerprint_sha256,
            created.meta.fingerprint_sha256
        );
        assert_eq!(load_active(&root).unwrap().meta.name, "copy");
    }
}
