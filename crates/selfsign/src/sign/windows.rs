use std::fs;
use std::process::Command;

use crate::identity::IdentityRecord;

use super::pfx::{export_pfx, random_password, temp_pfx_path};
use super::tools::find_signtool;
use super::{DiscoveredArtifact, SignOptions, SignReport, SignedArtifact};

pub fn sign_artifacts(
    identity: &IdentityRecord,
    artifacts: &[DiscoveredArtifact],
    opts: &SignOptions,
) -> anyhow::Result<SignReport> {
    let mut report = SignReport::default();
    let Some(signtool) = find_signtool() else {
        anyhow::bail!(
            "signtool.exe not found — install Windows SDK Signing Tools, or add signtool to PATH"
        );
    };

    let password = random_password();
    let pfx = temp_pfx_path(identity);
    export_pfx(identity, &password, &pfx)?;

    let signable: Vec<_> = artifacts
        .iter()
        .filter(|a| {
            matches!(
                a.kind,
                super::ArtifactKind::WindowsExe | super::ArtifactKind::WindowsMsi
            )
        })
        .collect();

    if signable.is_empty() {
        report.warnings.push(
            "no Windows .exe/.msi artifacts found to sign (did tauri build produce bundles?)"
                .into(),
        );
        let _ = fs::remove_file(&pfx);
        return Ok(report);
    }

    for art in signable {
        match sign_one(&signtool, &pfx, &password, &art.path, opts) {
            Ok(method) => {
                report.signed.push(SignedArtifact {
                    path: art.path.clone(),
                    kind: art.kind,
                    method,
                    note: Some(
                        "SmartScreen may still warn for self-signed / low-reputation certs".into(),
                    ),
                });
            }
            Err(err) => {
                report
                    .skipped
                    .push((art.path.clone(), format!("signtool failed: {err}")));
            }
        }
    }

    let _ = fs::remove_file(&pfx);
    Ok(report)
}

fn sign_one(
    signtool: &std::path::Path,
    pfx: &std::path::Path,
    password: &str,
    file: &std::path::Path,
    opts: &SignOptions,
) -> anyhow::Result<String> {
    if opts.timestamp {
        let status = Command::new(signtool)
            .args(["sign", "/fd", "SHA256", "/td", "SHA256", "/tr"])
            .arg(&opts.timestamp_url)
            .arg("/f")
            .arg(pfx)
            .arg("/p")
            .arg(password)
            .arg(file)
            .status()?;
        if status.success() {
            return Ok("signtool+timestamp".into());
        }
        // Fall back without timestamp (common offline / blocked TSA)
    }

    let status = Command::new(signtool)
        .args(["sign", "/fd", "SHA256", "/f"])
        .arg(pfx)
        .arg("/p")
        .arg(password)
        .arg(file)
        .status()?;

    if !status.success() {
        anyhow::bail!("exit status {status}");
    }
    Ok(if opts.timestamp {
        "signtool (timestamp unavailable; signed without TSA)".into()
    } else {
        "signtool".into()
    })
}
