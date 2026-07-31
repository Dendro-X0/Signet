use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::paths::{managed_binary_path, receipt_path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallMethod {
    Installer,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstallReceipt {
    pub method: InstallMethod,
    pub repo: String,
    pub installed_version: String,
    pub binary_path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

pub fn read_receipt() -> Option<InstallReceipt> {
    let path = receipt_path();
    let text = fs::read_to_string(path).ok()?;
    toml::from_str(&text).ok()
}

pub fn write_receipt(rec: &InstallReceipt) -> anyhow::Result<()> {
    let path = receipt_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bin_parent = managed_binary_path().parent().map(|p| p.to_path_buf());
    if let Some(dir) = bin_parent {
        fs::create_dir_all(dir)?;
    }
    let body = toml::to_string_pretty(rec)?;
    fs::write(path, format!("# Signet CLI install receipt — do not edit\n{body}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_round_trip_toml() {
        let rec = InstallReceipt {
            method: InstallMethod::Installer,
            repo: "Dendro-X0/Signet".into(),
            installed_version: "0.2.0".into(),
            binary_path: PathBuf::from("/tmp/signet"),
            updated_at: None,
        };
        let text = toml::to_string_pretty(&rec).unwrap();
        let parsed: InstallReceipt = toml::from_str(&text).unwrap();
        assert_eq!(rec, parsed);
    }
}
