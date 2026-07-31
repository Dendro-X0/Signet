use std::fs;
use std::path::Path;
use std::process::Command;

/// Replace the managed executable with `new_bytes`.
///
/// On Windows, renames the running image aside then writes the new file (allowed while running).
pub fn replace_executable(dest: &Path, new_bytes: &[u8]) -> anyhow::Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp = dest.with_extension("new");
    fs::write(&tmp, new_bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tmp)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tmp, perms)?;
    }

    if dest.exists() {
        let bak = dest.with_extension("old");
        let _ = fs::remove_file(&bak);
        fs::rename(dest, &bak)?;
        if let Err(e) = fs::rename(&tmp, dest) {
            let _ = fs::rename(&bak, dest);
            return Err(e.into());
        }
        let _ = fs::remove_file(&bak);
    } else {
        fs::rename(&tmp, dest)?;
    }
    Ok(())
}

/// Best-effort: remove leftover `.old` after a prior Windows update.
pub fn schedule_windows_cleanup(dest: &Path) {
    let bak = dest.with_extension("old");
    if bak.exists() {
        let _ = fs::remove_file(&bak);
    }
}

/// After uninstall, on Windows spawn a helper that deletes the running exe if needed.
pub fn windows_deferred_delete(path: &Path) -> anyhow::Result<()> {
    if !cfg!(windows) {
        return Ok(());
    }
    let path_s = path.display().to_string();
    // cmd delay then delete — process must exit soon after.
    let _ = Command::new("cmd")
        .args([
            "/C",
            "ping",
            "127.0.0.1",
            "-n",
            "2",
            ">",
            "nul",
            "&",
            "del",
            "/F",
            "/Q",
            &path_s,
        ])
        .spawn();
    Ok(())
}
