//! Shared artifact record for discover → sign → sums → release.

use std::path::PathBuf;

use super::ArtifactKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub path: PathBuf,
    pub kind: ArtifactKind,
    /// Basename (or unique release asset name) used in SHA256SUMS / GitHub.
    pub name_for_sums: String,
}

impl Artifact {
    pub fn new(path: PathBuf, kind: ArtifactKind) -> Self {
        let name_for_sums = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("artifact")
            .to_string();
        Self {
            path,
            kind,
            name_for_sums,
        }
    }

    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let kind = ArtifactKind::classify_explicit(&path);
        Self::new(path, kind)
    }
}

/// Compact JSON array for agents / future `--json` discover.
pub fn artifacts_json(artifacts: &[Artifact]) -> String {
    let mut out = String::from("[");
    for (i, a) in artifacts.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let path = a
            .path
            .to_string_lossy()
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        let name = a.name_for_sums.replace('\\', "\\\\").replace('"', "\\\"");
        out.push_str(&format!(
            "{{\"path\":\"{path}\",\"kind\":\"{}\",\"name_for_sums\":\"{name}\"}}",
            a.kind.as_str()
        ));
    }
    out.push(']');
    out
}

/// Filter artifacts host-signable on the current OS.
pub fn host_signable(artifacts: &[Artifact]) -> Vec<Artifact> {
    let os = std::env::consts::OS;
    artifacts
        .iter()
        .filter(|a| a.kind.host_signable_on(os))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn name_defaults_to_basename() {
        let a = Artifact::from_path(Path::new("dist/app-setup.exe"));
        assert_eq!(a.name_for_sums, "app-setup.exe");
        assert_eq!(a.kind, ArtifactKind::WindowsExe);
    }

    #[test]
    fn artifacts_json_round_shape() {
        let a = Artifact::new(PathBuf::from("a.exe"), ArtifactKind::WindowsExe);
        let j = artifacts_json(&[a]);
        assert!(j.contains("windows-exe"));
        assert!(j.contains("a.exe"));
    }
}
