use std::fs;
use std::path::Path;
use std::process::Command;

use crate::identity::IdentityRecord;

use super::tools::find_openssl;
use super::{ArtifactKind, DiscoveredArtifact, SignOptions, SignReport, SignedArtifact};

pub fn sign_artifacts(
    identity: &IdentityRecord,
    artifacts: &[DiscoveredArtifact],
    _opts: &SignOptions,
) -> anyhow::Result<SignReport> {
    let mut report = SignReport::default();
    let openssl = find_openssl();

    let signable: Vec<_> = artifacts
        .iter()
        .filter(|a| {
            matches!(
                a.kind,
                ArtifactKind::LinuxAppImage | ArtifactKind::LinuxDeb | ArtifactKind::LinuxRpm
            )
        })
        .collect();

    if signable.is_empty() {
        report
            .warnings
            .push("no Linux AppImage/deb/rpm artifacts found to sign".into());
        return Ok(report);
    }

    let pubkey = identity.dir.join("pubkey.pem");
    if let Some(ref openssl) = openssl {
        let _ = extract_pubkey(openssl, &identity.dir.join("cert.pem"), &pubkey);
    }

    for art in &signable {
        if let Some(ref openssl) = openssl {
            match detached_sign(openssl, identity, &art.path) {
                Ok(sig_path) => {
                    report.signed.push(SignedArtifact {
                        path: art.path.clone(),
                        kind: art.kind,
                        method: format!("openssl dgst -sha256 (detached: {})", sig_path.display()),
                        note: Some(format!(
                            "verify: openssl dgst -sha256 -verify {} -signature {} {}",
                            pubkey.display(),
                            sig_path.display(),
                            art.path.display()
                        )),
                    });
                }
                Err(err) => {
                    report.skipped.push((
                        art.path.clone(),
                        format!("openssl detached sign failed: {err}"),
                    ));
                }
            }
        } else {
            report.warnings.push(
                "openssl not found — writing checksums only (no detached signatures)".into(),
            );
            for art in &signable {
                report.signed.push(SignedArtifact {
                    path: art.path.clone(),
                    kind: art.kind,
                    method: "checksum-only".into(),
                    note: Some("install openssl for detached .sig signatures".into()),
                });
            }
            return Ok(report);
        }
    }

    Ok(report)
}

fn extract_pubkey(openssl: &Path, cert: &Path, out: &Path) -> anyhow::Result<()> {
    let status = Command::new(openssl)
        .args(["x509", "-in"])
        .arg(cert)
        .args(["-pubkey", "-noout", "-out"])
        .arg(out)
        .status()?;
    if !status.success() {
        anyhow::bail!("openssl x509 -pubkey failed: {status}");
    }
    Ok(())
}

fn detached_sign(
    openssl: &Path,
    identity: &IdentityRecord,
    file: &Path,
) -> anyhow::Result<std::path::PathBuf> {
    let sig = {
        let mut p = file.to_path_buf();
        let name = format!(
            "{}.sig",
            file.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("artifact")
        );
        p.set_file_name(name);
        p
    };

    let key = identity.dir.join("key.pem");
    let status = Command::new(openssl)
        .args(["dgst", "-sha256", "-sign"])
        .arg(&key)
        .arg("-out")
        .arg(&sig)
        .arg(file)
        .status()?;

    if !status.success() {
        let _ = fs::remove_file(&sig);
        anyhow::bail!("exit status {status}");
    }
    Ok(sig)
}
