use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::tools::find_openssl;
use crate::identity::IdentityRecord;

/// Export identity PEM material to a temporary PKCS#12 (.pfx) for signtool.
pub fn export_pfx(identity: &IdentityRecord, password: &str, out: &Path) -> anyhow::Result<()> {
    let openssl = find_openssl().ok_or_else(|| {
        anyhow::anyhow!(
            "openssl not found on PATH (required to export PFX from PEM for Windows signing)"
        )
    })?;

    let cert = identity.dir.join("cert.pem");
    let key = identity.dir.join("key.pem");
    if !cert.exists() || !key.exists() {
        anyhow::bail!("identity PEM files missing under {}", identity.dir.display());
    }

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }

    let status = Command::new(&openssl)
        .args([
            "pkcs12",
            "-export",
            "-inkey",
        ])
        .arg(&key)
        .arg("-in")
        .arg(&cert)
        .arg("-out")
        .arg(out)
        .arg("-passout")
        .arg(format!("pass:{password}"))
        .arg("-name")
        .arg(&identity.meta.common_name)
        .status()?;

    if !status.success() {
        anyhow::bail!("openssl pkcs12 export failed with status {status}");
    }
    Ok(())
}

pub fn random_password() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("signet-{nanos}-{}", std::process::id())
}

pub fn temp_pfx_path(identity: &IdentityRecord) -> PathBuf {
    identity.dir.join("signing-temp.pfx")
}
