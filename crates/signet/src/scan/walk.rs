use std::fs;
use std::path::{Path, PathBuf};

use super::report::{
    finalize_report, DetectedInstaller, DetectedProject, Platform, ProjectKind, ScanReport,
};

const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    ".signet",
    ".selfsign",
    ".next",
    "coverage",
    "__pycache__",
    ".venv",
    "venv",
    "incremental",
    ".fingerprint",
];

/// Cargo profile subdirs that are never shipping installers.
const SKIP_TARGET_SUBDIRS: &[&str] = &["build", "deps", "incremental", ".fingerprint", "examples"];

const MAX_DEPTH: usize = 8;
const MAX_INSTALLERS: usize = 80;

pub fn scan_repository(root: &Path) -> anyhow::Result<ScanReport> {
    let root = root
        .canonicalize()
        .unwrap_or_else(|_| root.to_path_buf());

    let mut projects = Vec::new();
    let mut installers = Vec::new();

    detect_projects(&root, &root, 0, &mut projects)?;
    walk_installers(&root, &root, 0, &mut installers)?;

    // Prefer deeper / more specific project entries; keep unique by path
    projects.sort_by(|a, b| a.path.cmp(&b.path));
    projects.dedup_by(|a, b| a.path == b.path && a.kind == b.kind);

    installers.sort_by(|a, b| a.path.cmp(&b.path));
    installers.dedup_by(|a, b| a.path == b.path);

    Ok(finalize_report(root, projects, installers))
}

fn detect_projects(
    _root: &Path,
    dir: &Path,
    depth: usize,
    out: &mut Vec<DetectedProject>,
) -> anyhow::Result<()> {
    if depth > MAX_DEPTH {
        return Ok(());
    }

    // Tauri: src-tauri/tauri.conf.json or tauri.conf.json here
    let tauri_conf = dir.join("src-tauri").join("tauri.conf.json");
    let tauri_here = dir.join("tauri.conf.json");
    if tauri_conf.is_file() {
        let name = read_tauri_product_name(&tauri_conf).or_else(|| read_pkg_name(dir));
        let mut detail = "Tauri desktop project".to_string();
        let gen_android = dir.join("src-tauri/gen/android").is_dir();
        let gen_apple = dir.join("src-tauri/gen/apple").is_dir()
            || dir.join("src-tauri/gen/ios").is_dir();
        if gen_android {
            detail.push_str("; android gen/ present");
        }
        if gen_apple {
            detail.push_str("; ios/apple gen/ present");
        }
        out.push(DetectedProject {
            kind: ProjectKind::Tauri,
            path: dir.to_path_buf(),
            name,
            detail,
        });
    } else if tauri_here.is_file() {
        // Prefer the app root that owns `src-tauri/`; skip the crate dir itself.
        let is_src_tauri = dir
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n == "src-tauri")
            .unwrap_or(false);
        if !is_src_tauri {
            let name = read_tauri_product_name(&tauri_here);
            out.push(DetectedProject {
                kind: ProjectKind::Tauri,
                path: dir.to_path_buf(),
                name,
                detail: "Tauri crate (tauri.conf.json)".into(),
            });
        }
    }

    // Electron
    let pkg = dir.join("package.json");
    let mut pkg_text = String::new();
    let has_pkg = if pkg.is_file() {
        match fs::read_to_string(&pkg) {
            Ok(t) => {
                pkg_text = t;
                true
            }
            Err(_) => false,
        }
    } else {
        false
    };

    // Expo before React Native (expo apps also list react-native)
    if has_pkg && (pkg_text.contains("\"expo\"") || pkg_text.contains("\"expo-")) {
        let name = json_string_field(&pkg_text, "name");
        out.push(DetectedProject {
            kind: ProjectKind::Expo,
            path: dir.to_path_buf(),
            name,
            detail: "Expo markers in package.json".into(),
        });
    } else if has_pkg
        && (pkg_text.contains("\"react-native\"") || pkg_text.contains("react-native/"))
    {
        let name = json_string_field(&pkg_text, "name");
        out.push(DetectedProject {
            kind: ProjectKind::ReactNative,
            path: dir.to_path_buf(),
            name,
            detail: "React Native markers in package.json".into(),
        });
    }

    if has_pkg
        && (pkg_text.contains("\"electron\"")
            || pkg_text.contains("electron-builder")
            || pkg_text.contains("electron-forge"))
    {
        let name = json_string_field(&pkg_text, "name");
        out.push(DetectedProject {
            kind: ProjectKind::Electron,
            path: dir.to_path_buf(),
            name,
            detail: "Electron packaging markers in package.json".into(),
        });
    }

    // Capacitor
    let cap_config = dir.join("capacitor.config.ts").is_file()
        || dir.join("capacitor.config.json").is_file()
        || dir.join("capacitor.config.js").is_file();
    if cap_config
        || (has_pkg
            && (pkg_text.contains("@capacitor/core") || pkg_text.contains("\"@capacitor/")))
    {
        let name = if has_pkg {
            json_string_field(&pkg_text, "name")
        } else {
            None
        };
        out.push(DetectedProject {
            kind: ProjectKind::Capacitor,
            path: dir.to_path_buf(),
            name,
            detail: "Capacitor project markers".into(),
        });
    }

    // Flutter
    let pubspec = dir.join("pubspec.yaml");
    if pubspec.is_file() {
        if let Ok(text) = fs::read_to_string(&pubspec) {
            if text.contains("flutter:")
                || text.contains("sdk: flutter")
                || text.contains("sdk:flutter")
            {
                let name = text.lines().find_map(|l| {
                    let t = l.trim();
                    t.strip_prefix("name:")
                        .map(|v| v.trim().trim_matches('"').to_string())
                });
                out.push(DetectedProject {
                    kind: ProjectKind::Flutter,
                    path: dir.to_path_buf(),
                    name,
                    detail: "Flutter pubspec.yaml".into(),
                });
            }
        }
    }

    // Android native / Capacitor
    if (dir.join("build.gradle").is_file() || dir.join("build.gradle.kts").is_file())
        && (dir.join("src/main/AndroidManifest.xml").is_file()
            || dir.join("app/src/main/AndroidManifest.xml").is_file())
    {
        out.push(DetectedProject {
            kind: ProjectKind::AndroidNative,
            path: dir.to_path_buf(),
            name: None,
            detail: "Android Gradle project".into(),
        });
    }

    // iOS
    if has_extension_dir(dir, "xcodeproj") || has_extension_dir(dir, "xcworkspace") {
        out.push(DetectedProject {
            kind: ProjectKind::IosNative,
            path: dir.to_path_buf(),
            name: None,
            detail: "Xcode project / workspace".into(),
        });
    }

    for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if should_skip_dir(&name) {
            continue;
        }
        // Still descend into target for installers only via walk_installers; skip heavy target for project detect
        if name == "target" {
            continue;
        }
        detect_projects(_root, &path, depth + 1, out)?;
    }
    Ok(())
}

fn walk_installers(
    _root: &Path,
    dir: &Path,
    depth: usize,
    out: &mut Vec<DetectedInstaller>,
) -> anyhow::Result<()> {
    if depth > MAX_DEPTH || out.len() >= MAX_INSTALLERS {
        return Ok(());
    }

    let dir_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let under_target = path_norm(dir).contains("/target/");
    let is_profile = under_target && matches!(dir_name.as_str(), "debug" | "release" | "bench");

    for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            if should_skip_dir(&name) {
                continue;
            }
            if is_profile && SKIP_TARGET_SUBDIRS.iter().any(|s| name.eq_ignore_ascii_case(s)) {
                continue;
            }
            // Under target/, only care about bundle/ (and profile roots we already entered)
            if under_target && !is_profile {
                let n = name.to_ascii_lowercase();
                // Allow: target → debug/release, profile → bundle, bundle → *
                let parent = dir
                    .file_name()
                    .and_then(|p| p.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let parent_is_target = parent == "target";
                if parent_is_target && !matches!(n.as_str(), "debug" | "release") {
                    continue;
                }
                if matches!(parent.as_str(), "debug" | "release") && n != "bundle" {
                    // already handled via SKIP_TARGET_SUBDIRS; double-safe
                    if SKIP_TARGET_SUBDIRS.iter().any(|s| n == *s) {
                        continue;
                    }
                    if n != "bundle" {
                        continue;
                    }
                }
            }
            if name.ends_with(".app") {
                if is_distributable(&path) {
                    push_installer(
                        out,
                        path,
                        Platform::Macos,
                        "app",
                        "codesign / Gatekeeper (self-sign ≠ notarized)",
                    );
                }
                continue;
            }
            walk_installers(_root, &path, depth + 1, out)?;
            continue;
        }

        if let Some(inst) = classify_installer(&path) {
            if is_distributable(&inst.path) {
                out.push(inst);
            }
        }
    }
    Ok(())
}

fn path_norm(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_ascii_lowercase()
}

/// Drop Cargo build-script / deps noise; keep bundle outputs and real packages.
fn is_distributable(path: &Path) -> bool {
    let norm = path_norm(path);
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if name.contains("build-script") {
        return false;
    }
    if norm.contains("/target/") {
        if norm.contains("/build/")
            || norm.contains("/deps/")
            || norm.contains("/incremental/")
            || norm.contains("/.fingerprint/")
        {
            return false;
        }
        // Prefer packaged bundles; allow main release binary only (no nested path junk)
        if norm.contains("/bundle/") {
            return true;
        }
        // e.g. target/release/app.exe — one segment after release
        if let Some(idx) = norm.rfind("/target/") {
            let rest = &norm[idx + "/target/".len()..];
            let parts: Vec<_> = rest.split('/').collect();
            // release/foo.exe — main binary only (skip debug noise)
            if parts.len() == 2 && parts[0] == "release" {
                return matches!(
                    path.extension().and_then(|e| e.to_str()).unwrap_or(""),
                    "exe"
                ) || name.ends_with(".appimage");
            }
        }
        return false;
    }
    true
}

fn classify_installer(path: &Path) -> Option<DetectedInstaller> {
    let name = path.file_name()?.to_str()?.to_ascii_lowercase();

    if name.ends_with(".appimage") {
        return Some(installer(
            path,
            Platform::Linux,
            "AppImage",
            "openssl detached sig + SHA256SUMS",
        ));
    }

    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "exe" | "msi" | "msix" => Some(installer(
            path,
            Platform::Windows,
            &ext,
            "Authenticode via signtool (SmartScreen may warn)",
        )),
        "dmg" | "pkg" => Some(installer(
            path,
            Platform::Macos,
            &ext,
            "codesign (notarization requires Apple Developer)",
        )),
        "deb" | "rpm" => Some(installer(
            path,
            Platform::Linux,
            &ext,
            "openssl detached sig + SHA256SUMS",
        )),
        "apk" | "aab" => Some(installer(
            path,
            Platform::Android,
            &ext,
            "listed only — use Android / Play signing tooling",
        )),
        "ipa" => Some(installer(
            path,
            Platform::Ios,
            "ipa",
            "listed only — use Apple certificates / notarization where required",
        )),
        _ => None,
    }
}

fn installer(
    path: &Path,
    platform: Platform,
    format: &str,
    signed_hint: &str,
) -> DetectedInstaller {
    DetectedInstaller {
        platform,
        path: path.to_path_buf(),
        format: format.into(),
        signed_hint: signed_hint.into(),
    }
}

fn push_installer(
    out: &mut Vec<DetectedInstaller>,
    path: PathBuf,
    platform: Platform,
    format: &str,
    signed_hint: &str,
) {
    if out.len() >= MAX_INSTALLERS {
        return;
    }
    out.push(DetectedInstaller {
        platform,
        path,
        format: format.into(),
        signed_hint: signed_hint.into(),
    });
}

fn should_skip_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
        || (name.starts_with('.') && name != ".signet" && name != ".selfsign")
}

fn has_extension_dir(dir: &Path, ext: &str) -> bool {
    fs::read_dir(dir)
        .ok()
        .map(|rd| {
            rd.flatten().any(|e| {
                e.path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|x| x == ext)
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn read_pkg_name(dir: &Path) -> Option<String> {
    let text = fs::read_to_string(dir.join("package.json")).ok()?;
    json_string_field(&text, "name")
}

fn read_tauri_product_name(conf: &Path) -> Option<String> {
    let text = fs::read_to_string(conf).ok()?;
    // Tauri v1: package.productName; v2: productName / identifier
    json_string_field(&text, "productName")
        .or_else(|| nested_json_string(&text, "package", "productName"))
        .or_else(|| json_string_field(&text, "identifier"))
}

fn json_string_field(text: &str, key: &str) -> Option<String> {
    // Minimal scanner — avoids pulling serde_json schema for arbitrary JSON shapes.
    let needle = format!("\"{key}\"");
    let idx = text.find(&needle)?;
    let after = &text[idx + needle.len()..];
    let after = after.trim_start().trim_start_matches(':').trim_start();
    if !after.starts_with('"') {
        return None;
    }
    let rest = &after[1..];
    let end = rest.find('"')?;
    let val = &rest[..end];
    if val.is_empty() {
        None
    } else {
        Some(val.to_string())
    }
}

fn nested_json_string(text: &str, parent: &str, key: &str) -> Option<String> {
    let parent_needle = format!("\"{parent}\"");
    let idx = text.find(&parent_needle)?;
    let slice = &text[idx..];
    // Search within a limited window after parent
    let window = &slice[..slice.len().min(800)];
    json_string_field(window, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn finds_tauri_and_windows_installer() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("src-tauri")).unwrap();
        fs::write(
            root.join("src-tauri/tauri.conf.json"),
            r#"{ "productName": "DemoApp", "identifier": "com.demo.app" }"#,
        )
        .unwrap();
        let bundle = root.join("src-tauri/target/release/bundle/nsis");
        fs::create_dir_all(&bundle).unwrap();
        fs::write(bundle.join("DemoApp_0.1.0_x64-setup.exe"), b"MZ").unwrap();
        fs::create_dir_all(root.join("android-out")).unwrap();
        fs::write(root.join("android-out/app-release.apk"), b"apk").unwrap();

        let report = scan_repository(root).unwrap();
        assert!(report
            .projects
            .iter()
            .any(|p| p.kind == ProjectKind::Tauri));
        assert!(report
            .installers
            .iter()
            .any(|i| i.platform == Platform::Windows));
        assert!(report
            .installers
            .iter()
            .any(|i| i.platform == Platform::Android));
        assert!(report.suggested.windows);
        assert!(report.suggested.android);
        assert!(!report.next_steps.is_empty());
    }

    #[test]
    fn ignores_cargo_build_script_exes() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let noise = root.join("target/debug/build/foo-hash");
        fs::create_dir_all(&noise).unwrap();
        fs::write(noise.join("build-script-build.exe"), b"MZ").unwrap();
        let deps = root.join("target/debug/deps");
        fs::create_dir_all(&deps).unwrap();
        fs::write(deps.join("libfoo.exe"), b"MZ").unwrap();

        fs::create_dir_all(root.join("src-tauri")).unwrap();
        fs::write(
            root.join("src-tauri/tauri.conf.json"),
            r#"{ "productName": "Real" }"#,
        )
        .unwrap();
        let bundle = root.join("src-tauri/target/release/bundle/nsis");
        fs::create_dir_all(&bundle).unwrap();
        fs::write(bundle.join("Real-setup.exe"), b"MZ").unwrap();

        let report = scan_repository(root).unwrap();
        assert_eq!(
            report
                .installers
                .iter()
                .filter(|i| i.platform == Platform::Windows)
                .count(),
            1
        );
        assert!(report
            .installers
            .iter()
            .any(|i| i.path.ends_with("Real-setup.exe")));
    }
}
