//! Detect a project version for release-tag defaults (guided + scan suggestions).

use std::fs;
use std::path::Path;
use std::process::Command;

/// Best-effort project version string **without** requiring a leading `v`.
pub fn detect_project_version(root: &Path) -> Option<String> {
    cargo_version(root)
        .or_else(|| package_json_version(root))
        .or_else(|| app_json_version(root))
        .or_else(|| git_describe_tag(root))
}

/// Default git tag for prompts / CLI hints (always non-empty, with leading `v`).
pub fn default_release_tag(root: &Path) -> String {
    let ver = detect_project_version(root).unwrap_or_else(|| "0.1.0".into());
    ensure_v_prefix(&ver)
}

pub fn ensure_v_prefix(version: &str) -> String {
    let t = version.trim().trim_start_matches(['v', 'V']);
    if t.is_empty() {
        "v0.1.0".into()
    } else {
        format!("v{t}")
    }
}

fn cargo_version(root: &Path) -> Option<String> {
    let text = fs::read_to_string(root.join("Cargo.toml")).ok()?;
    cargo_toml_version(&text)
}

fn cargo_toml_version(text: &str) -> Option<String> {
    if let Some(v) = toml_section_string(text, "[package]", "version") {
        return Some(v);
    }
    toml_section_string(text, "[workspace.package]", "version")
}

fn toml_section_string(text: &str, section: &str, key: &str) -> Option<String> {
    let mut in_section = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_section = t == section;
            continue;
        }
        if !in_section {
            continue;
        }
        let Some(rest) = t.strip_prefix(key) else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let val = rest.trim().trim_matches('"').trim_matches('\'').to_string();
        if !val.is_empty() {
            return Some(val);
        }
    }
    None
}

fn package_json_version(root: &Path) -> Option<String> {
    let text = fs::read_to_string(root.join("package.json")).ok()?;
    json_top_level_string(&text, "version")
}

fn app_json_version(root: &Path) -> Option<String> {
    for name in ["app.json", "app.config.json"] {
        let path = root.join(name);
        if !path.is_file() {
            continue;
        }
        let text = fs::read_to_string(path).ok()?;
        if let Some(v) = json_top_level_string(&text, "version") {
            return Some(v);
        }
        if let Some(v) = nested_json_string_field(&text, "expo", "version") {
            return Some(v);
        }
    }
    None
}

fn json_top_level_string(text: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let mut search = text;
    while let Some(idx) = search.find(&needle) {
        let after = &search[idx + needle.len()..];
        let after = after.trim_start();
        if !after.starts_with(':') {
            search = &search[idx + 1..];
            continue;
        }
        let after = after[1..].trim_start();
        if let Some(s) = json_quoted(after) {
            // Skip nested-looking hits deep in file if we already skipped — take first quoted string.
            if !s.is_empty() {
                return Some(s);
            }
        }
        search = &search[idx + 1..];
    }
    None
}

fn nested_json_string_field(text: &str, parent: &str, key: &str) -> Option<String> {
    let parent_needle = format!("\"{parent}\"");
    let idx = text.find(&parent_needle)?;
    let window = &text[idx..text.len().min(idx + 1200)];
    json_top_level_string(window, key)
}

fn json_quoted(s: &str) -> Option<String> {
    let s = s.trim_start();
    if !s.starts_with('"') {
        return None;
    }
    let mut out = String::new();
    let mut chars = s.chars().skip(1);
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => {
                if let Some(n) = chars.next() {
                    out.push(n);
                }
            }
            _ => out.push(c),
        }
    }
    None
}

fn git_describe_tag(root: &Path) -> Option<String> {
    let out = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let tag = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if tag.is_empty() {
        None
    } else {
        Some(tag.trim_start_matches(['v', 'V']).to_string())
    }
}

/// Compare config framework id to scan suggestion (`rn` ≡ `react-native`).
pub fn frameworks_equivalent(a: &str, b: &str) -> bool {
    normalize_fw(a) == normalize_fw(b)
}

fn normalize_fw(s: &str) -> String {
    let s = s.trim().to_ascii_lowercase();
    match s.as_str() {
        "rn" | "react_native" => "react-native".into(),
        "rust" | "rust-cli" => "cli".into(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn ensure_v_prefix_adds_and_keeps() {
        assert_eq!(ensure_v_prefix("0.5.3"), "v0.5.3");
        assert_eq!(ensure_v_prefix("v1.2.0"), "v1.2.0");
        assert_eq!(ensure_v_prefix("V2.0.0"), "v2.0.0");
    }

    #[test]
    fn detects_workspace_cargo_version() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            r#"[workspace]
members = ["crates/x"]

[workspace.package]
version = "0.5.3"
edition = "2021"
"#,
        )
        .unwrap();
        assert_eq!(
            detect_project_version(dir.path()).as_deref(),
            Some("0.5.3")
        );
        assert_eq!(default_release_tag(dir.path()), "v0.5.3");
    }

    #[test]
    fn detects_package_cargo_over_workspace() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            r#"[package]
name = "app"
version = "1.2.3"
edition = "2021"

[workspace.package]
version = "9.9.9"
"#,
        )
        .unwrap();
        assert_eq!(
            detect_project_version(dir.path()).as_deref(),
            Some("1.2.3")
        );
    }

    #[test]
    fn detects_package_json_version() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{ "name": "app", "version": "3.1.4" }"#,
        )
        .unwrap();
        assert_eq!(
            detect_project_version(dir.path()).as_deref(),
            Some("3.1.4")
        );
        assert_eq!(default_release_tag(dir.path()), "v3.1.4");
    }

    #[test]
    fn detects_expo_app_json_version() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("app.json"),
            r#"{ "expo": { "name": "App", "version": "2.0.1" } }"#,
        )
        .unwrap();
        assert_eq!(
            detect_project_version(dir.path()).as_deref(),
            Some("2.0.1")
        );
    }

    #[test]
    fn frameworks_rn_alias() {
        assert!(frameworks_equivalent("rn", "react-native"));
        assert!(!frameworks_equivalent("tauri", "cli"));
    }

    #[test]
    fn fallback_tag() {
        let dir = tempdir().unwrap();
        assert_eq!(default_release_tag(dir.path()), "v0.1.0");
    }
}
