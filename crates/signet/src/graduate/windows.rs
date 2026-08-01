//! Windows OV Authenticode + Azure Trusted Signing (signtool wrappers).

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::sign::find_signtool;

use super::honesty_notes;

const DEFAULT_OV_TIMESTAMP: &str = "http://timestamp.digicert.com";
const DEFAULT_AZURE_TIMESTAMP: &str = "http://timestamp.acs.microsoft.com";

#[derive(Debug, Clone)]
pub enum OvCredential {
    Thumbprint(String),
    Pfx { path: PathBuf, password: String },
}

#[derive(Debug, Clone)]
pub struct OvSignOptions {
    pub credential: OvCredential,
    pub timestamp_url: String,
    pub timestamp: bool,
}

impl Default for OvSignOptions {
    fn default() -> Self {
        Self {
            credential: OvCredential::Thumbprint(String::new()),
            timestamp_url: DEFAULT_OV_TIMESTAMP.into(),
            timestamp: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AzureSignOptions {
    pub dlib: PathBuf,
    pub metadata: PathBuf,
    pub timestamp_url: String,
}

impl Default for AzureSignOptions {
    fn default() -> Self {
        Self {
            dlib: PathBuf::new(),
            metadata: PathBuf::new(),
            timestamp_url: DEFAULT_AZURE_TIMESTAMP.into(),
        }
    }
}

/// Build `signtool sign …` argv after the executable path (for tests / dry inspection).
pub fn build_ov_sign_argv(opts: &OvSignOptions, file: &Path) -> anyhow::Result<Vec<String>> {
    match &opts.credential {
        OvCredential::Thumbprint(tp) if tp.trim().is_empty() => {
            anyhow::bail!(
                "OV thumbprint required — pass --thumbprint, SIGNET_OV_THUMBPRINT, or [graduation].ov_thumbprint \
                 (or use --pfx / SIGNET_OV_PFX). Will not fall back to Signet self-signed identity."
            );
        }
        OvCredential::Pfx { path, password } => {
            if !path.is_file() {
                anyhow::bail!("OV PFX not found: {}", path.display());
            }
            if password.is_empty() {
                anyhow::bail!("SIGNET_OV_PFX_PASS (or --pfx-pass) required when using a PFX");
            }
        }
        _ => {}
    }

    let mut args = vec!["sign".into(), "/fd".into(), "SHA256".into()];
    if opts.timestamp {
        args.extend([
            "/td".into(),
            "SHA256".into(),
            "/tr".into(),
            opts.timestamp_url.clone(),
        ]);
    }
    match &opts.credential {
        OvCredential::Thumbprint(tp) => {
            let cleaned: String = tp.chars().filter(|c| !c.is_whitespace()).collect();
            args.extend(["/sha1".into(), cleaned]);
        }
        OvCredential::Pfx { path, password } => {
            args.extend([
                "/f".into(),
                path.display().to_string(),
                "/p".into(),
                password.clone(),
            ]);
        }
    }
    args.push(file.display().to_string());
    Ok(args)
}

pub fn build_azure_sign_argv(opts: &AzureSignOptions, file: &Path) -> anyhow::Result<Vec<String>> {
    if opts.dlib.as_os_str().is_empty() || !opts.dlib.is_file() {
        anyhow::bail!(
            "Azure Code Signing dlib required — set [graduation.azure].dlib or SIGNET_AZURE_DLIB \
             (see docs/graduation.md)"
        );
    }
    if opts.metadata.as_os_str().is_empty() || !opts.metadata.is_file() {
        anyhow::bail!(
            "Azure Trusted Signing metadata JSON required — set [graduation.azure].metadata or \
             SIGNET_AZURE_METADATA"
        );
    }
    Ok(vec![
        "sign".into(),
        "/fd".into(),
        "SHA256".into(),
        "/td".into(),
        "SHA256".into(),
        "/tr".into(),
        opts.timestamp_url.clone(),
        "/dlib".into(),
        opts.dlib.display().to_string(),
        "/dmdf".into(),
        opts.metadata.display().to_string(),
        file.display().to_string(),
    ])
}

pub fn ov_sign_files(files: &[PathBuf], opts: &OvSignOptions) -> anyhow::Result<Vec<PathBuf>> {
    let signtool = find_signtool().ok_or_else(|| {
        anyhow::anyhow!("signtool.exe not found — install Windows SDK Signing Tools")
    })?;
    let mut signed = Vec::new();
    for file in files {
        if !file.is_file() {
            anyhow::bail!("file not found: {}", file.display());
        }
        let args = build_ov_sign_argv(opts, file)?;
        let status = Command::new(&signtool).args(&args).status()?;
        if !status.success() {
            anyhow::bail!("signtool ov-sign failed for {}: {status}", file.display());
        }
        signed.push(file.clone());
    }
    eprintln!("note: {}", honesty_notes());
    eprintln!(
        "hint: if this cert chains to a public CA, set trust.declared_tier = \"ca_authenticode\" in signet.toml"
    );
    Ok(signed)
}

pub fn azure_sign_files(files: &[PathBuf], opts: &AzureSignOptions) -> anyhow::Result<Vec<PathBuf>> {
    let signtool = find_signtool().ok_or_else(|| {
        anyhow::anyhow!("signtool.exe not found — install Windows SDK Signing Tools")
    })?;
    let mut signed = Vec::new();
    for file in files {
        if !file.is_file() {
            anyhow::bail!("file not found: {}", file.display());
        }
        let args = build_azure_sign_argv(opts, file)?;
        let status = Command::new(&signtool).args(&args).status()?;
        if !status.success() {
            anyhow::bail!(
                "signtool azure-sign failed for {} ({status}). Ensure Azure identity is configured for the dlib — see docs/graduation.md",
                file.display()
            );
        }
        signed.push(file.clone());
    }
    eprintln!("note: {}", honesty_notes());
    eprintln!(
        "hint: after Trusted Signing succeeds, set trust.declared_tier = \"ca_authenticode\" when appropriate"
    );
    Ok(signed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn ov_thumbprint_argv() {
        let opts = OvSignOptions {
            credential: OvCredential::Thumbprint("AB CD".into()),
            timestamp_url: DEFAULT_OV_TIMESTAMP.into(),
            timestamp: true,
        };
        let args = build_ov_sign_argv(&opts, Path::new("app.exe")).unwrap();
        assert!(args.contains(&"/sha1".into()));
        assert!(args.contains(&"ABCD".into()));
        assert!(args.contains(&"app.exe".into()));
        assert!(!args.iter().any(|a| a.contains("signet")));
    }

    #[test]
    fn ov_rejects_empty_thumbprint() {
        let opts = OvSignOptions::default();
        let err = build_ov_sign_argv(&opts, Path::new("app.exe")).unwrap_err();
        assert!(err.to_string().contains("thumbprint"));
    }

    #[test]
    fn azure_argv_with_fixture_files() {
        let dir = tempfile::tempdir().unwrap();
        let dlib = dir.path().join("Azure.CodeSigning.Dlib.dll");
        let meta = dir.path().join("metadata.json");
        fs_write(&dlib, b"dll");
        fs_write(&meta, b"{}");
        let opts = AzureSignOptions {
            dlib,
            metadata: meta,
            timestamp_url: DEFAULT_AZURE_TIMESTAMP.into(),
        };
        let args = build_azure_sign_argv(&opts, Path::new("app.exe")).unwrap();
        assert!(args.contains(&"/dlib".into()));
        assert!(args.contains(&"/dmdf".into()));
        assert!(args.contains(&DEFAULT_AZURE_TIMESTAMP.into()));
    }

    fn fs_write(path: &Path, bytes: &[u8]) {
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(bytes).unwrap();
    }
}
