use std::path::PathBuf;
use std::process::Command;

use which::which;

pub fn find_openssl() -> Option<PathBuf> {
    which("openssl").ok()
}

pub fn find_codesign() -> Option<PathBuf> {
    which("codesign").ok()
}

pub fn find_tauri_cli() -> Option<TauriCli> {
    if which("cargo-tauri").is_ok() {
        return Some(TauriCli::CargoTauri);
    }
    if which("tauri").is_ok() {
        return Some(TauriCli::Tauri);
    }
    // `cargo tauri` works when the cargo subcommand is installed
    if Command::new("cargo")
        .args(["tauri", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some(TauriCli::CargoTauri);
    }
    None
}

#[derive(Debug, Clone, Copy)]
pub enum TauriCli {
    CargoTauri,
    Tauri,
}

impl TauriCli {
    pub fn run_build(&self, src_tauri: &std::path::Path, extra: &[String]) -> anyhow::Result<()> {
        let status = match self {
            Self::CargoTauri => Command::new("cargo")
                .current_dir(src_tauri)
                .arg("tauri")
                .arg("build")
                .args(extra)
                .status()?,
            Self::Tauri => Command::new("tauri")
                .current_dir(src_tauri)
                .arg("build")
                .args(extra)
                .status()?,
        };
        if !status.success() {
            anyhow::bail!("tauri build failed with status {status}");
        }
        Ok(())
    }
}

pub fn find_signtool() -> Option<PathBuf> {
    if let Ok(path) = which("signtool") {
        return Some(path);
    }

    let bases = [
        r"C:\Program Files (x86)\Windows Kits\10\bin",
        r"C:\Program Files\Windows Kits\10\bin",
    ];
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x86"
    };

    for base in bases {
        let base = PathBuf::from(base);
        if !base.is_dir() {
            continue;
        }
        let mut versions = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&base) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    versions.push(p);
                }
            }
        }
        versions.sort();
        versions.reverse();
        for ver in versions {
            let candidate = ver.join(arch).join("signtool.exe");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}
