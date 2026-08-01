use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Windows,
    Macos,
    Linux,
    Android,
    Ios,
}

impl Platform {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Macos => "macos",
            Self::Linux => "linux",
            Self::Android => "android",
            Self::Ios => "ios",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    Tauri,
    Electron,
    AndroidNative,
    IosNative,
    Flutter,
    ReactNative,
    Expo,
    Capacitor,
    /// Rust binary / workspace without a desktop/mobile UI app adapter.
    RustCli,
}

impl ProjectKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tauri => "tauri",
            Self::Electron => "electron",
            Self::AndroidNative => "android",
            Self::IosNative => "ios",
            Self::Flutter => "flutter",
            Self::ReactNative => "react_native",
            Self::Expo => "expo",
            Self::Capacitor => "capacitor",
            Self::RustCli => "rust_cli",
        }
    }

    /// True when this is a shippable UI/mobile app stack (not a plain CLI).
    pub fn is_installable_app(self) -> bool {
        !matches!(self, Self::RustCli)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DetectedProject {
    pub kind: ProjectKind,
    pub path: PathBuf,
    pub name: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DetectedInstaller {
    pub platform: Platform,
    pub path: PathBuf,
    pub format: String,
    pub signed_hint: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuggestedConfig {
    pub project_name: String,
    pub tauri_root: String,
    /// Suggested `[project].framework` (e.g. tauri, electron, cli).
    pub framework: String,
    pub windows: bool,
    pub macos: bool,
    pub linux: bool,
    /// Mobile targets are detected for awareness; desktop self-sign path does not cover store signing.
    pub android: bool,
    pub ios: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NextStep {
    pub command: String,
    pub why: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanReport {
    pub root: PathBuf,
    pub projects: Vec<DetectedProject>,
    pub installers: Vec<DetectedInstaller>,
    pub suggested: SuggestedConfig,
    pub next_steps: Vec<NextStep>,
    pub has_signet: bool,
    pub has_identity: bool,
}

pub fn print_human(report: &ScanReport) {
    use crate::ui::console::{self, display_path};

    console::banner("scan");
    console::kv(14, "root", &short_root(&report.root));

    console::section("projects");
    if report.projects.is_empty() {
        console::muted(
            "none detected — looking for Tauri / Electron / Flutter / RN / Expo / Capacitor / \
             Android / iOS / Rust CLI markers",
        );
    } else {
        for p in &report.projects {
            let name = p.name.as_deref().unwrap_or("?");
            let rel = display_path(&report.root, &p.path);
            console::bullet(&format!(
                "[{}] {name}  {rel}  — {}",
                p.kind.as_str(),
                p.detail
            ));
        }
    }

    console::section("installers");
    if report.installers.is_empty() {
        console::muted("none yet — run a platform build, then re-scan");
    } else {
        const MAX_SHOW: usize = 5;
        for plat in [
            Platform::Windows,
            Platform::Macos,
            Platform::Linux,
            Platform::Android,
            Platform::Ios,
        ] {
            let items: Vec<_> = report
                .installers
                .iter()
                .filter(|i| i.platform == plat)
                .collect();
            if items.is_empty() {
                continue;
            }
            console::platform_header(plat.as_str(), items.len());
            for i in items.iter().take(MAX_SHOW) {
                let rel = display_path(&report.root, &i.path);
                console::bullet(&format!("{rel}  ({}) — {}", i.format, i.signed_hint));
            }
            if items.len() > MAX_SHOW {
                console::muted(&format!("{} more omitted", items.len() - MAX_SHOW));
            }
        }
    }

    console::section("suggested config");
    let s = &report.suggested;
    const W: usize = 16;
    console::kv(W, "project.name", &format!("{:?}", s.project_name));
    console::kv(W, "tauri_root", &format!("{:?}", s.tauri_root));
    console::kv(W, "framework", &format!("{:?}", s.framework));
    console::kv(
        W,
        "platforms",
        &format!(
            "windows={}  macos={}  linux={}",
            s.windows, s.macos, s.linux
        ),
    );
    console::kv(
        W,
        "mobile",
        &format!("android={}  ios={}", s.android, s.ios),
    );
    for note in &s.notes {
        console::note(note);
    }

    console::section("status");
    console::status(16, "signet.toml", report.has_signet, "");
    console::status(16, "identity", report.has_identity, "");
    console::status(
        16,
        "desktop installers",
        report.installers.iter().any(|i| {
            matches!(
                i.platform,
                Platform::Windows | Platform::Macos | Platform::Linux
            )
        }),
        "",
    );

    console::section("next steps");
    for (i, step) in report.next_steps.iter().enumerate() {
        console::numbered(i + 1, &step.command, &step.why);
    }
    console::blank();
}

fn short_root(root: &Path) -> String {
    root.file_name()
        .and_then(|n| n.to_str())
        .map(|n| format!("./{n}"))
        .unwrap_or_else(|| root.display().to_string())
}

/// Rank projects for config suggestions: shallow paths win; demo/fixture paths lose.
pub fn preferred_project<'a>(
    root: &Path,
    projects: &'a [DetectedProject],
) -> Option<&'a DetectedProject> {
    const ORDER: &[ProjectKind] = &[
        ProjectKind::Tauri,
        ProjectKind::Electron,
        ProjectKind::Flutter,
        ProjectKind::Expo,
        ProjectKind::ReactNative,
        ProjectKind::Capacitor,
        ProjectKind::AndroidNative,
        ProjectKind::IosNative,
        ProjectKind::RustCli,
    ];
    projects.iter().min_by_key(|p| {
        let kind_rank = ORDER
            .iter()
            .position(|k| *k == p.kind)
            .unwrap_or(ORDER.len());
        (path_preference_score(root, &p.path), kind_rank)
    })
}

/// Lower is better. Depth + penalties for sample/demo trees.
pub fn path_preference_score(root: &Path, path: &Path) -> usize {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let mut score = rel.components().count() * 10;
    for comp in rel.components() {
        let Some(s) = comp.as_os_str().to_str() else {
            continue;
        };
        match s.to_ascii_lowercase().as_str() {
            "demo" | "demos" | "fixture" | "fixtures" | "example" | "examples" | "testdata"
            | "samples" | "sample" => score += 100,
            _ => {}
        }
    }
    score
}

/// Map a detected kind to `[project].framework`.
pub fn framework_id_for_kind(kind: ProjectKind) -> &'static str {
    match kind {
        ProjectKind::Tauri => "tauri",
        ProjectKind::Electron => "electron",
        ProjectKind::Flutter => "flutter",
        ProjectKind::ReactNative => "react-native",
        ProjectKind::Expo => "expo",
        ProjectKind::Capacitor => "capacitor",
        ProjectKind::AndroidNative => "android",
        ProjectKind::IosNative => "ios",
        ProjectKind::RustCli => "cli",
    }
}

/// Build suggested config + next steps from raw detections.
pub fn finalize_report(
    root: PathBuf,
    projects: Vec<DetectedProject>,
    installers: Vec<DetectedInstaller>,
) -> ScanReport {
    let has_signet = root.join("signet.toml").is_file() || root.join("selfsign.toml").is_file();
    let has_identity = root.join(".signet/identity/active").is_file()
        || root.join(".selfsign/identity/active").is_file();

    let preferred = preferred_project(&root, &projects);
    let tauri = projects.iter().find(|p| p.kind == ProjectKind::Tauri);
    let project_name = preferred
        .and_then(|p| p.name.clone())
        .or_else(|| {
            projects.iter().find_map(|p| p.name.clone()).or_else(|| {
                root.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
        })
        .unwrap_or_else(|| "my-app".into());

    let tauri_root = preferred
        .map(|p| relativize(&root, &p.path))
        .or_else(|| tauri.map(|p| relativize(&root, &p.path)))
        .unwrap_or_else(|| ".".into());

    let framework = preferred
        .map(|p| framework_id_for_kind(p.kind).to_string())
        .unwrap_or_else(|| "tauri".into());

    let mut windows = installers.iter().any(|i| i.platform == Platform::Windows);
    let mut macos = installers.iter().any(|i| i.platform == Platform::Macos);
    let mut linux = installers.iter().any(|i| i.platform == Platform::Linux);
    let android = installers.iter().any(|i| i.platform == Platform::Android)
        || projects.iter().any(|p| p.kind == ProjectKind::AndroidNative)
        || projects.iter().any(|p| {
            p.kind == ProjectKind::Tauri && p.detail.to_ascii_lowercase().contains("android")
        });
    let ios = installers.iter().any(|i| i.platform == Platform::Ios)
        || projects.iter().any(|p| p.kind == ProjectKind::IosNative)
        || projects.iter().any(|p| {
            p.kind == ProjectKind::Tauri && p.detail.to_ascii_lowercase().contains("ios")
        });

    // If Tauri desktop project with no artifacts yet, default all three desktop platforms on.
    if tauri.is_some() && !windows && !macos && !linux {
        windows = true;
        macos = true;
        linux = true;
    }
    // CLI tooling: host binary only — don't imply a full desktop installer matrix.
    if framework == "cli" && !windows && !macos && !linux {
        match std::env::consts::OS {
            "windows" => windows = true,
            "macos" => macos = true,
            "linux" => linux = true,
            _ => windows = true,
        }
    }

    let mut notes = Vec::new();
    if framework == "cli" {
        notes.push(
            "Detected a Rust CLI / workspace (not an installable desktop/mobile app). \
             Use framework = \"cli\" to build and sign the host binary, or point init at an app subfolder."
                .into(),
        );
    } else {
        notes.push(
            "Desktop self-signing (Windows/macOS/Linux) is what `signet build` covers today. \
             Reputation ladder (OV / Azure / notarize): `signet graduate notes` + docs/graduation.md."
                .into(),
        );
    }
    if android || ios {
        notes.push(
            "Mobile installers detected — Android: `signet android` + docs/android.md; \
             iOS: `signet ios package` + docs/ios.md (free provisioning ~7 days; no App Store trust)."
                .into(),
        );
    }
    if android && framework != "cli" {
        notes.push(
            "Android: set framework = \"android\" for APK discover/sign, or run \
             `signet android sign --apk …` after keystore create."
                .into(),
        );
    }
    if preferred.map(|p| p.kind) == Some(ProjectKind::Electron)
        || (framework != "cli" && projects.iter().any(|p| p.kind == ProjectKind::Electron))
    {
        if framework == "electron" {
            notes.push(
                "Electron app detected — set [project].framework = \"electron\" in signet.toml, \
                 then `signet build` (or --skip-build) to sign/checksum installers under dist/out/release."
                    .into(),
            );
        }
    }
    if preferred.map(|p| p.kind.is_installable_app()).unwrap_or(false)
        && projects.iter().any(|p| {
            matches!(
                p.kind,
                ProjectKind::Flutter
                    | ProjectKind::ReactNative
                    | ProjectKind::Expo
                    | ProjectKind::Capacitor
            )
        })
        && matches!(
            framework.as_str(),
            "flutter" | "react-native" | "expo" | "capacitor"
        )
    {
        notes.push(
            "Hybrid framework detected — set framework to flutter|react-native|expo|capacitor, \
             set build_command, see docs/frameworks.md. APK: signet android sign; IPA: signet ios package."
                .into(),
        );
    }

    let mut next_steps = Vec::new();
    if !has_signet {
        next_steps.push(NextStep {
            command: format!(
                "signet init --name {} --tauri-root {} --framework {}",
                shell_quote(&project_name),
                shell_quote(&tauri_root),
                shell_quote(&framework)
            ),
            why: "create signet.toml from this suggestion".into(),
        });
    }
    if !has_identity {
        next_steps.push(NextStep {
            command: "signet identity create".into(),
            why: "create a local code-signing identity".into(),
        });
    }
    if has_identity && !root.join("TRUST.md").is_file() {
        next_steps.push(NextStep {
            command: "signet trust".into(),
            why: "emit honest install / fingerprint docs".into(),
        });
    }
    if framework == "cli" {
        next_steps.push(NextStep {
            command: "signet build".into(),
            why: "cargo build --release and sign the host binary".into(),
        });
    } else if !installers
        .iter()
        .any(|i| matches!(i.platform, Platform::Windows | Platform::Macos | Platform::Linux))
    {
        next_steps.push(NextStep {
            command: "signet build".into(),
            why: "produce and sign desktop installers".into(),
        });
    } else {
        next_steps.push(NextStep {
            command: "signet build --skip-build".into(),
            why: "sign the installers already on disk".into(),
        });
        next_steps.push(NextStep {
            command: "signet release --tag v0.1.0 --dry-run".into(),
            why: "prepare GitHub Release assets".into(),
        });
    }
    next_steps.push(NextStep {
        command: "signet".into(),
        why: "open the TUI Guided setup if you prefer prompts".into(),
    });

    ScanReport {
        root,
        projects,
        installers,
        suggested: SuggestedConfig {
            project_name,
            tauri_root,
            framework,
            windows,
            macos,
            linux,
            android,
            ios,
            notes,
        },
        next_steps,
        has_signet,
        has_identity,
    }
}

fn relativize(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| {
            let s = p.to_string_lossy().replace('\\', "/");
            if s.is_empty() {
                ".".into()
            } else {
                s
            }
        })
        .unwrap_or_else(|_| ".".into())
}

fn shell_quote(s: &str) -> String {
    if s.chars()
        .any(|c| c.is_whitespace() || matches!(c, '"' | '\''))
    {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.to_string()
    }
}
