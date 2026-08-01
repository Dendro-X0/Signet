//! Installer-managed CLI install: paths, receipt, update, uninstall.

mod github;
mod paths;
mod receipt;
mod replace;

pub use github::{download_bytes, expected_sha256_from_sums, fetch_latest_release};
pub use paths::{install_root, managed_binary_path, receipt_path};
pub use receipt::{read_receipt, write_receipt, InstallMethod, InstallReceipt};
pub use replace::{replace_executable, schedule_windows_cleanup, windows_deferred_delete};

use std::fs;
use std::path::PathBuf;

use paths::is_under_install_root;

#[derive(Debug, Clone)]
pub struct InstallStatus {
    pub managed: bool,
    pub method: String,
    pub version: String,
    pub binary: PathBuf,
    pub receipt_path: Option<PathBuf>,
    pub detail: String,
}

pub fn current_status() -> InstallStatus {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("signet"));
    let receipt_path = paths::receipt_path();
    let running_version = env!("CARGO_PKG_VERSION").to_string();

    if let Some(rec) = read_receipt() {
        if rec.method == InstallMethod::Installer {
            let managed = same_exe(&exe, &rec.binary_path) || is_under_install_root(&exe);
            let detail = if managed {
                "installer-managed — `signet self update` / `uninstall` available".into()
            } else if is_cargo_install_path(&exe) {
                format!(
                    "WARNING: this process is cargo (~/.cargo/bin) while an installer \
                     receipt exists for {} (v{}). PATH is shadowed — run `cargo uninstall signet` \
                     or invoke the managed binary directly, then open a new terminal",
                    rec.binary_path.display(),
                    rec.installed_version
                )
            } else {
                format!(
                    "receipt found for {} (v{}) but this process is not that binary — PATH may be shadowed",
                    rec.binary_path.display(),
                    rec.installed_version
                )
            };
            return InstallStatus {
                managed,
                method: if managed {
                    "installer".into()
                } else if is_cargo_install_path(&exe) {
                    "cargo".into()
                } else {
                    "installer".into()
                },
                version: if managed {
                    rec.installed_version.clone()
                } else {
                    running_version
                },
                binary: exe,
                receipt_path: Some(receipt_path),
                detail,
            };
        }
    }

    let method = if is_cargo_install_path(&exe) {
        "cargo"
    } else if is_under_install_root(&exe) {
        "installer" // binary in place but receipt missing
    } else {
        "dev"
    };

    InstallStatus {
        managed: method == "installer" && is_under_install_root(&exe),
        method: method.into(),
        version: running_version,
        binary: exe,
        receipt_path: receipt_path.exists().then_some(receipt_path),
        detail: match method {
            "cargo" => "installed via cargo — update with `cargo install --force --git …` or the official installer".into(),
            "installer" => "under install root but receipt missing — re-run installer".into(),
            _ => "development / local build — use installer for self-update".into(),
        },
    }
}

fn same_exe(a: &std::path::Path, b: &std::path::Path) -> bool {
    let ca = fs::canonicalize(a).unwrap_or_else(|_| a.to_path_buf());
    let cb = fs::canonicalize(b).unwrap_or_else(|_| b.to_path_buf());
    ca == cb
}

fn is_cargo_install_path(exe: &std::path::Path) -> bool {
    let s = exe.to_string_lossy();
    s.contains(".cargo/bin") || s.contains(".cargo\\bin")
}

/// Ensure receipt exists for a freshly copied managed binary (used by update).
pub fn ensure_managed_receipt(version: &str) -> anyhow::Result<InstallReceipt> {
    let binary = managed_binary_path();
    let rec = InstallReceipt {
        method: InstallMethod::Installer,
        repo: "Dendro-X0/Signet".into(),
        installed_version: version.into(),
        binary_path: binary,
        updated_at: Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default(),
        ),
    };
    write_receipt(&rec)?;
    Ok(rec)
}
