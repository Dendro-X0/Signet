use std::path::{Path, PathBuf};

/// User-level install root for the Signet CLI (not per-app `.signet/`).
pub fn install_root() -> PathBuf {
    if cfg!(windows) {
        dirs_local_data().join("Signet")
    } else {
        dirs_home().join(".signet-cli")
    }
}

pub fn managed_binary_path() -> PathBuf {
    let name = if cfg!(windows) { "signet.exe" } else { "signet" };
    install_root().join("bin").join(name)
}

pub fn receipt_path() -> PathBuf {
    install_root().join("install.toml")
}

/// Optional Windows mirror under `%USERPROFILE%\bin\signet.exe` for Git Bash / Cursor PATH.
pub fn windows_home_shim_path() -> Option<PathBuf> {
    if !cfg!(windows) {
        return None;
    }
    Some(dirs_home().join("bin").join("signet.exe"))
}

/// Keep `%USERPROFILE%\bin\signet.exe` in sync with the managed binary (best-effort).
pub fn sync_windows_home_shim() {
    let Some(shim) = windows_home_shim_path() else {
        return;
    };
    let src = managed_binary_path();
    if !src.is_file() {
        return;
    }
    if let Some(parent) = shim.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::copy(&src, &shim);
}

/// Remove the home shim if present (uninstall).
pub fn remove_windows_home_shim() {
    if let Some(shim) = windows_home_shim_path() {
        let _ = std::fs::remove_file(shim);
    }
}

pub fn is_under_install_root(path: &Path) -> bool {
    let root = install_root();
    let Ok(canon_root) = std::fs::canonicalize(&root) else {
        // Root may not exist yet
        let s = path.to_string_lossy();
        let r = root.to_string_lossy();
        return s.contains(r.as_ref());
    };
    let Ok(canon_path) = std::fs::canonicalize(path) else {
        return false;
    };
    canon_path.starts_with(canon_root)
}

/// GitHub Release asset filename for this host.
pub fn host_asset_name() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "signet-x86_64-pc-windows-msvc.exe",
        ("linux", "x86_64") => "signet-x86_64-unknown-linux-gnu",
        ("macos", "x86_64") => "signet-x86_64-apple-darwin",
        ("macos", "aarch64") => "signet-aarch64-apple-darwin",
        ("linux", "aarch64") => "signet-aarch64-unknown-linux-gnu",
        _ => "signet-unknown",
    }
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn dirs_local_data() -> PathBuf {
    if let Some(p) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(p);
    }
    dirs_home().join("AppData").join("Local")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_name_known_for_windows_or_unix() {
        let name = host_asset_name();
        assert!(name.starts_with("signet-"));
        assert_ne!(name, "signet-unknown");
    }
}
