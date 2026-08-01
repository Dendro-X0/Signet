//! Locate Android SDK / JDK signing tools.

use std::fs;
use std::path::PathBuf;

use which::which;

pub fn find_keytool() -> Option<PathBuf> {
    which("keytool").ok()
}

pub fn find_jarsigner() -> Option<PathBuf> {
    which("jarsigner").ok()
}

pub fn find_apksigner() -> Option<PathBuf> {
    if let Ok(p) = which("apksigner") {
        return Some(p);
    }
    // Windows build-tools ship apksigner.bat
    if let Ok(p) = which("apksigner.bat") {
        return Some(p);
    }
    find_apksigner_in_sdk()
}

fn sdk_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for key in ["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
        if let Ok(v) = std::env::var(key) {
            let p = PathBuf::from(v);
            if p.is_dir() {
                roots.push(p);
            }
        }
    }
    roots
}

fn find_apksigner_in_sdk() -> Option<PathBuf> {
    for root in sdk_roots() {
        let bt = root.join("build-tools");
        if !bt.is_dir() {
            continue;
        }
        let mut versions: Vec<_> = fs::read_dir(&bt)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        versions.sort();
        versions.reverse(); // prefer newest
        for ver in versions {
            for name in ["apksigner.bat", "apksigner"] {
                let cand = ver.join(name);
                if cand.is_file() {
                    return Some(cand);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_roots_reads_env() {
        // Smoke: function does not panic without env.
        let _ = sdk_roots();
        let _ = find_keytool();
        let _ = find_apksigner();
    }
}
