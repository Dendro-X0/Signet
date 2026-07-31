//! Optional host integration: sign a tiny PE when signtool + openssl exist.

#[cfg(all(test, windows))]
mod windows_sign {
    use std::fs;
    use std::process::Command;

    use crate::identity::{create_identity, CreateOptions};
    use crate::sign::{
        find_openssl, find_signtool, sign_host_artifacts, ArtifactKind, DiscoveredArtifact,
        SignOptions,
    };

    #[test]
    fn signs_pe_when_tools_present() {
        if find_signtool().is_none() || find_openssl().is_none() {
            eprintln!("skip: signtool/openssl not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let id_root = dir.path().join("identity");
        let identity = create_identity(
            &id_root,
            &CreateOptions {
                name: "default".into(),
                common_name: "Selfsign Test".into(),
                organization: "Test".into(),
                days: 30,
                force: false,
            },
        )
        .unwrap();

        // Compile a tiny PE with rustc
        let src = dir.path().join("tiny.rs");
        fs::write(&src, "fn main() {}").unwrap();
        let exe = dir.path().join("tiny.exe");
        let status = Command::new("rustc")
            .arg(&src)
            .arg("-o")
            .arg(&exe)
            .status()
            .unwrap();
        assert!(status.success());

        let artifacts = vec![DiscoveredArtifact {
            path: exe.clone(),
            kind: ArtifactKind::WindowsExe,
        }];
        let report = sign_host_artifacts(
            &identity,
            &artifacts,
            &SignOptions {
                timestamp: false,
                ..SignOptions::default()
            },
        )
        .unwrap();
        assert_eq!(report.signed.len(), 1, "skipped: {:?}", report.skipped);
    }
}
