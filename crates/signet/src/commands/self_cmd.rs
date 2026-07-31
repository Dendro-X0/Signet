//! `signet self` — status / update / uninstall for installer-managed CLI copies.

use std::fs;

use clap::{Args as ClapArgs, Subcommand};
use sha2::{Digest, Sha256};

use crate::self_manage::{
    current_status, download_bytes, ensure_managed_receipt, expected_sha256_from_sums,
    fetch_latest_release, install_root, managed_binary_path, receipt_path, replace_executable,
    schedule_windows_cleanup, windows_deferred_delete, write_receipt, InstallMethod,
    InstallReceipt,
};
use crate::ui::console;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: SelfCommand,
}

#[derive(Debug, Subcommand)]
pub enum SelfCommand {
    /// Show how this Signet binary was installed
    Status,
    /// Download the latest GitHub Release binary (installer-managed)
    Update {
        /// Only report whether an update is available
        #[arg(long)]
        check: bool,
        /// Attempt in-place replace even if not installer-managed (dangerous)
        #[arg(long)]
        force: bool,
    },
    /// Remove the installer-managed CLI binary and receipt
    Uninstall {
        #[arg(long)]
        yes: bool,
    },
}

pub fn run(args: Args) -> anyhow::Result<()> {
    match args.command {
        SelfCommand::Status => status(),
        SelfCommand::Update { check, force } => update(check, force),
        SelfCommand::Uninstall { yes } => uninstall(yes),
    }
}

fn status() -> anyhow::Result<()> {
    schedule_windows_cleanup(&managed_binary_path());
    let st = current_status();
    console::banner("self status");
    console::kv(16, "version", &st.version);
    console::kv(16, "method", &st.method);
    console::kv(16, "managed", if st.managed { "yes" } else { "no" });
    console::kv(16, "binary", &st.binary.display().to_string());
    if let Some(p) = &st.receipt_path {
        console::kv(16, "receipt", &p.display().to_string());
    }
    console::blank();
    console::note(&st.detail);
    console::blank();
    Ok(())
}

fn update(check_only: bool, force: bool) -> anyhow::Result<()> {
    console::banner("self update");
    let st = current_status();
    if !st.managed && !force {
        anyhow::bail!(
            "this Signet is not installer-managed ({})\n\
             Install with the one-liner in the README, or pass --force to replace this binary anyway",
            st.detail
        );
    }

    console::kv(16, "current", &st.version);
    let release = fetch_latest_release()?;
    console::kv(16, "latest", &release.tag);
    console::kv(16, "asset", &release.asset_name);

    if release.version == st.version.trim_start_matches('v') && !force {
        console::ok_line("already up to date");
        return Ok(());
    }

    if check_only {
        console::ok_line(&format!(
            "update available: {} → {}",
            st.version, release.tag
        ));
        return Ok(());
    }

    console::note(&format!("downloading {} …", release.asset_name));
    let bytes = download_bytes(&release.download_url)?;
    if let Some(sums_url) = &release.sums_url {
        let sums = String::from_utf8(download_bytes(sums_url)?)
            .map_err(|e| anyhow::anyhow!("SHA256SUMS not utf8: {e}"))?;
        if let Some(expected) = expected_sha256_from_sums(&sums, &release.asset_name) {
            let actual = hex::encode(Sha256::digest(&bytes));
            if actual != expected {
                anyhow::bail!(
                    "checksum mismatch for {} (expected {expected}, got {actual})",
                    release.asset_name
                );
            }
            console::ok_line("SHA256SUMS verified");
        } else {
            console::note("asset not listed in SHA256SUMS — skipping checksum");
        }
    } else {
        console::note("no SHA256SUMS on release — skipping checksum");
    }

    let dest = if st.managed {
        managed_binary_path()
    } else {
        st.binary.clone()
    };

    replace_executable(&dest, &bytes)?;

    let rec = InstallReceipt {
        method: InstallMethod::Installer,
        repo: "Dendro-X0/Signet".into(),
        installed_version: release.version.clone(),
        binary_path: dest.clone(),
        updated_at: Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default(),
        ),
    };
    if dest == managed_binary_path() {
        let _ = ensure_managed_receipt(&release.version);
        write_receipt(&rec)?;
    } else {
        write_receipt(&rec)?;
    }

    console::ok_line(&format!("updated to {} ({})", release.tag, dest.display()));
    console::note("restart Signet to run the new binary");
    console::blank();
    Ok(())
}

fn uninstall(yes: bool) -> anyhow::Result<()> {
    console::banner("self uninstall");
    let st = current_status();
    if !st.managed {
        anyhow::bail!(
            "refusing to uninstall — not installer-managed ({})\n\
             Remove cargo/dev builds yourself (e.g. `cargo uninstall signet`)",
            st.detail
        );
    }

    if !yes {
        console::note(&format!("will remove {}", st.binary.display()));
        console::note("re-run with --yes to confirm (TUI confirms interactively)");
        anyhow::bail!("uninstall not confirmed — pass --yes");
    }

    let bin = st.binary.clone();
    let receipt = receipt_path();

    if bin.exists() {
        let doomed = bin.with_extension("uninstalled");
        let _ = fs::remove_file(&doomed);
        match fs::rename(&bin, &doomed) {
            Ok(()) => {
                if fs::remove_file(&doomed).is_err() {
                    windows_deferred_delete(&doomed)?;
                }
            }
            Err(_) => {
                windows_deferred_delete(&bin)?;
            }
        }
    }

    if receipt.exists() {
        fs::remove_file(&receipt)?;
    }

    if let Some(bin_dir) = managed_binary_path().parent() {
        let _ = fs::remove_dir(bin_dir);
    }
    let _ = fs::remove_dir(install_root());

    console::ok_line("Signet CLI removed from the install root");
    console::note("Remove the install bin directory from PATH if you added it manually");
    console::note("Project `.signet/` folders are untouched");
    console::blank();
    Ok(())
}

pub fn uninstall_confirmed() -> anyhow::Result<()> {
    uninstall(true)
}

pub fn update_default() -> anyhow::Result<()> {
    update(false, false)
}

pub fn status_public() -> anyhow::Result<()> {
    status()
}
