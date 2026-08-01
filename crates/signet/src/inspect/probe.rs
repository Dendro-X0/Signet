//! Platform signature probes.

use std::path::Path;
use std::process::Command;

use crate::android::find_apksigner;
use crate::artifact::ArtifactKind;
use crate::sign::{find_codesign, find_signtool};

use super::report::{platform_for_kind, InspectRow, SignatureStatus};

pub fn inspect_path(path: &Path) -> InspectRow {
    let kind = ArtifactKind::classify_explicit(path);
    let platform = platform_for_kind(kind).to_string();
    if !path.exists() {
        return InspectRow {
            path: path.to_path_buf(),
            kind: kind.as_str().into(),
            platform,
            status: SignatureStatus::Error,
            method: "fs".into(),
            detail: "path not found".into(),
        };
    }

    let (status, method, detail) = match kind {
        ArtifactKind::WindowsExe | ArtifactKind::WindowsMsi => probe_windows(path),
        ArtifactKind::MacApp | ArtifactKind::MacDmg => probe_macos(path),
        ArtifactKind::Apk => probe_android_apk(path),
        ArtifactKind::Aab => (
            SignatureStatus::Unknown,
            "aab".into(),
            "AAB signature inspect not implemented — use Play / jarsigner tooling".into(),
        ),
        ArtifactKind::LinuxAppImage | ArtifactKind::LinuxDeb | ArtifactKind::LinuxRpm => {
            probe_linux_detached(path)
        }
        ArtifactKind::Ipa => (
            SignatureStatus::Unknown,
            "ios-ipa".into(),
            "IPA host codesign inspect deferred — use a macOS .app + codesign, or Apple tooling"
                .into(),
        ),
        ArtifactKind::Zip | ArtifactKind::Other => (
            SignatureStatus::Unknown,
            "classify".into(),
            "unsupported or unknown artifact kind for signature inspect".into(),
        ),
    };

    InspectRow {
        path: path.to_path_buf(),
        kind: kind.as_str().into(),
        platform,
        status,
        method,
        detail,
    }
}

fn probe_windows(path: &Path) -> (SignatureStatus, String, String) {
    let Some(signtool) = find_signtool() else {
        return (
            SignatureStatus::Unknown,
            "signtool".into(),
            "signtool not found (install Windows SDK Signing Tools)".into(),
        );
    };
    match Command::new(&signtool)
        .args(["verify", "/pa"])
        .arg(path)
        .output()
    {
        Ok(out) if out.status.success() => (
            SignatureStatus::Signed,
            "signtool-verify".into(),
            "Authenticode signature present (not a SmartScreen guarantee)".into(),
        ),
        Ok(_) => (
            SignatureStatus::Unsigned,
            "signtool-verify".into(),
            "no Authenticode signature accepted by signtool verify /pa".into(),
        ),
        Err(e) => (
            SignatureStatus::Error,
            "signtool-verify".into(),
            format!("failed to run signtool: {e}"),
        ),
    }
}

fn probe_macos(path: &Path) -> (SignatureStatus, String, String) {
    let Some(codesign) = find_codesign() else {
        return (
            SignatureStatus::Unknown,
            "codesign".into(),
            if cfg!(target_os = "macos") {
                "codesign not found".into()
            } else {
                "codesign available on macOS hosts only".into()
            },
        );
    };

    let verify = Command::new(&codesign)
        .args(["--verify", "--verbose=2"])
        .arg(path)
        .output();
    match verify {
        Ok(out) if out.status.success() => {
            if macos_is_adhoc(&codesign, path) {
                (
                    SignatureStatus::Adhoc,
                    "codesign-verify".into(),
                    "ad-hoc signature (not Developer ID / notarization)".into(),
                )
            } else {
                (
                    SignatureStatus::Signed,
                    "codesign-verify".into(),
                    "codesign verify succeeded (not a Gatekeeper / notarization guarantee)".into(),
                )
            }
        }
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr);
            if err.to_ascii_lowercase().contains("code object is not signed")
                || err.to_ascii_lowercase().contains("not signed at all")
            {
                (
                    SignatureStatus::Unsigned,
                    "codesign-verify".into(),
                    "no codesign signature".into(),
                )
            } else {
                (
                    SignatureStatus::Unsigned,
                    "codesign-verify".into(),
                    truncate_detail(&err),
                )
            }
        }
        Err(e) => (
            SignatureStatus::Error,
            "codesign-verify".into(),
            format!("failed to run codesign: {e}"),
        ),
    }
}

fn macos_is_adhoc(codesign: &Path, path: &Path) -> bool {
    let Ok(out) = Command::new(codesign).args(["-dv"]).arg(path).output() else {
        return false;
    };
    // codesign -dv writes to stderr
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    parse_codesign_adhoc(&text)
}

/// Parse `codesign -dv` output for ad-hoc markers (unit-tested).
pub fn parse_codesign_adhoc(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("signature=adhoc")
        || lower.contains("authority=(adhoc)")
        || lower.lines().any(|l| {
            let t = l.trim();
            t.eq_ignore_ascii_case("authority=-") || t.eq_ignore_ascii_case("signature=adhoc")
        })
}

fn probe_android_apk(path: &Path) -> (SignatureStatus, String, String) {
    let Some(apksigner) = find_apksigner() else {
        return (
            SignatureStatus::Unknown,
            "apksigner".into(),
            "apksigner not found — install Android build-tools or use `signet android`".into(),
        );
    };
    match Command::new(&apksigner).args(["verify"]).arg(path).output() {
        Ok(out) if out.status.success() => (
            SignatureStatus::Signed,
            "apksigner-verify".into(),
            "APK signature present (local keystore ≠ Play App Signing key)".into(),
        ),
        Ok(_) => (
            SignatureStatus::Unsigned,
            "apksigner-verify".into(),
            "apksigner verify failed (unsigned or invalid)".into(),
        ),
        Err(e) => (
            SignatureStatus::Error,
            "apksigner-verify".into(),
            format!("failed to run apksigner: {e}"),
        ),
    }
}

fn probe_linux_detached(path: &Path) -> (SignatureStatus, String, String) {
    let sibling = sibling_sig(path);
    if sibling.is_file() {
        (
            SignatureStatus::Signed,
            "openssl-detached-sibling".into(),
            format!("found {}", sibling.display()),
        )
    } else {
        (
            SignatureStatus::Unsigned,
            "openssl-detached-sibling".into(),
            "no sibling .sig (checksums may still exist — use signet verify)".into(),
        )
    }
}

fn sibling_sig(path: &Path) -> std::path::PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(".sig");
    std::path::PathBuf::from(s)
}

fn truncate_detail(s: &str) -> String {
    let t = s.trim();
    if t.len() > 200 {
        format!("{}…", &t[..200])
    } else if t.is_empty() {
        "codesign verify failed".into()
    } else {
        t.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_adhoc_marker() {
        assert!(parse_codesign_adhoc("Signature=adhoc\nFormat=bundle"));
        assert!(!parse_codesign_adhoc("Authority=Developer ID Application: Example"));
    }

    #[test]
    fn linux_unsigned_without_sig() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("app.AppImage");
        std::fs::File::create(&app).unwrap().write_all(b"x").unwrap();
        let row = inspect_path(&app);
        assert_eq!(row.platform, "linux");
        assert_eq!(row.status, SignatureStatus::Unsigned);
    }

    #[test]
    fn linux_signed_with_sibling_sig() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("app.AppImage");
        std::fs::File::create(&app).unwrap().write_all(b"x").unwrap();
        let mut sig = app.as_os_str().to_owned();
        sig.push(".sig");
        std::fs::File::create(std::path::PathBuf::from(&sig))
            .unwrap()
            .write_all(b"sig")
            .unwrap();
        let row = inspect_path(&app);
        assert_eq!(row.status, SignatureStatus::Signed);
        assert_eq!(row.method, "openssl-detached-sibling");
    }

    #[test]
    fn missing_path_is_error() {
        let row = inspect_path(Path::new("definitely-missing-signet-inspect.bin"));
        assert_eq!(row.status, SignatureStatus::Error);
    }

    #[test]
    fn ipa_unknown() {
        let dir = tempfile::tempdir().unwrap();
        let ipa = dir.path().join("app.ipa");
        std::fs::File::create(&ipa).unwrap().write_all(b"zip").unwrap();
        let row = inspect_path(&ipa);
        assert_eq!(row.platform, "ios");
        assert_eq!(row.status, SignatureStatus::Unknown);
    }
}
