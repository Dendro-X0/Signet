use std::fs;
use std::process::Command;

use crate::identity::IdentityRecord;

use super::pfx::{export_pfx, random_password, temp_pfx_path};
use super::tools::find_codesign;
use super::{ArtifactKind, DiscoveredArtifact, SignOptions, SignReport, SignedArtifact};

pub fn sign_artifacts(
    identity: &IdentityRecord,
    artifacts: &[DiscoveredArtifact],
    _opts: &SignOptions,
) -> anyhow::Result<SignReport> {
    let mut report = SignReport::default();
    let Some(codesign) = find_codesign() else {
        anyhow::bail!("codesign not found (macOS only)");
    };

    let signable: Vec<_> = artifacts
        .iter()
        .filter(|a| matches!(a.kind, ArtifactKind::MacApp | ArtifactKind::MacDmg))
        .collect();

    if signable.is_empty() {
        report
            .warnings
            .push("no macOS .app/.dmg artifacts found to sign".into());
        return Ok(report);
    }

    // Prefer signing with a temporary keychain identity imported from PFX.
    // If import fails, fall back to ad-hoc (`-`) with an explicit warning.
    let identity_name = match import_temp_identity(identity) {
        Ok(name) => Some(name),
        Err(err) => {
            report.warnings.push(format!(
                "could not import identity into a temporary keychain ({err}); using ad-hoc codesign"
            ));
            None
        }
    };

    for art in signable {
        let sign_id = identity_name.as_deref().unwrap_or("-");
        let mut cmd = Command::new(&codesign);
        cmd.args(["--force", "--sign", sign_id]);
        if art.kind == ArtifactKind::MacApp {
            cmd.arg("--deep");
        }
        cmd.arg(&art.path);
        let status = cmd.status()?;
        if status.success() {
            let method = if identity_name.is_some() {
                "codesign (temp keychain identity)"
            } else {
                "codesign --sign - (ad-hoc)"
            };
            report.signed.push(SignedArtifact {
                path: art.path.clone(),
                kind: art.kind,
                method: method.into(),
                note: Some(
                    "Gatekeeper may still block; notarization requires an Apple Developer account"
                        .into(),
                ),
            });
        } else {
            report
                .skipped
                .push((art.path.clone(), format!("codesign failed: {status}")));
        }
    }

    Ok(report)
}

fn import_temp_identity(identity: &IdentityRecord) -> anyhow::Result<String> {
    let password = random_password();
    let pfx = temp_pfx_path(identity);
    export_pfx(identity, &password, &pfx)?;

    let keychain = identity.dir.join("signet-temp.keychain-db");
    let kc_pass = random_password();

    let status = Command::new("security")
        .args(["create-keychain", "-p"])
        .arg(&kc_pass)
        .arg(&keychain)
        .status()?;
    if !status.success() {
        let _ = fs::remove_file(&pfx);
        anyhow::bail!("create-keychain failed: {status}");
    }

    let _ = Command::new("security")
        .args(["set-keychain-settings", "-lut", "21600"])
        .arg(&keychain)
        .status();
    let _ = Command::new("security")
        .args(["unlock-keychain", "-p"])
        .arg(&kc_pass)
        .arg(&keychain)
        .status();

    let import = Command::new("security")
        .args(["import"])
        .arg(&pfx)
        .args(["-k"])
        .arg(&keychain)
        .args(["-P"])
        .arg(&password)
        .args(["-T", "/usr/bin/codesign", "-A"])
        .status()?;

    let _ = fs::remove_file(&pfx);

    if !import.success() {
        let _ = Command::new("security")
            .args(["delete-keychain"])
            .arg(&keychain)
            .status();
        anyhow::bail!("security import failed: {import}");
    }

    // Leave keychain in place for this signing session; best-effort cleanup later.
    // codesign looks up by common name when possible.
    Ok(identity.meta.common_name.clone())
}
