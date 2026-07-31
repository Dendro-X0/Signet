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
}

impl ProjectKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tauri => "tauri",
            Self::Electron => "electron",
            Self::AndroidNative => "android",
            Self::IosNative => "ios",
        }
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
    pub has_selfsign: bool,
    pub has_identity: bool,
}

pub fn print_human(report: &ScanReport) {
    use crate::ui::console::{self, display_path};

    console::banner("scan");
    console::kv(14, "root", &short_root(&report.root));

    console::section("projects");
    if report.projects.is_empty() {
        console::muted("none detected — looking for Tauri / Electron / Android / iOS markers");
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
            println!("  {}  ({} found)", plat.as_str(), items.len());
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
    console::status(16, "selfsign.toml", report.has_selfsign, "");
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

/// Build suggested config + next steps from raw detections.
pub fn finalize_report(
    root: PathBuf,
    projects: Vec<DetectedProject>,
    installers: Vec<DetectedInstaller>,
) -> ScanReport {
    let has_selfsign = root.join("selfsign.toml").is_file();
    let has_identity = root.join(".selfsign/identity/active").is_file();

    let tauri = projects.iter().find(|p| p.kind == ProjectKind::Tauri);
    let project_name = tauri
        .and_then(|p| p.name.clone())
        .or_else(|| {
            projects
                .iter()
                .find_map(|p| p.name.clone())
                .or_else(|| {
                    root.file_name()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
        })
        .unwrap_or_else(|| "my-app".into());

    let tauri_root = tauri
        .map(|p| relativize(&root, &p.path))
        .unwrap_or_else(|| ".".into());

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

    let mut notes = Vec::new();
    notes.push(
        "Desktop self-signing (Windows/macOS/Linux) is what `selfsign build` covers today.".into(),
    );
    if android || ios {
        notes.push(
            "Android/iOS installers were detected — store signing uses Play App Signing / Apple certificates; \
             selfsign lists them for awareness and does not claim to replace those programs."
                .into(),
        );
    }
    if projects.iter().any(|p| p.kind == ProjectKind::Electron) {
        notes.push(
            "Electron app detected — Phase 'later' may add non-Tauri signing; treat as experimental target."
                .into(),
        );
    }

    let mut next_steps = Vec::new();
    if !has_selfsign {
        next_steps.push(NextStep {
            command: format!(
                "selfsign init --name {} --tauri-root {}",
                shell_quote(&project_name),
                shell_quote(&tauri_root)
            ),
            why: "create selfsign.toml from this suggestion".into(),
        });
    }
    if !has_identity {
        next_steps.push(NextStep {
            command: "selfsign identity create".into(),
            why: "create a local code-signing identity".into(),
        });
    }
    if has_identity && !root.join("TRUST.md").is_file() {
        next_steps.push(NextStep {
            command: "selfsign trust".into(),
            why: "emit honest install / fingerprint docs".into(),
        });
    }
    if !installers
        .iter()
        .any(|i| matches!(i.platform, Platform::Windows | Platform::Macos | Platform::Linux))
    {
        next_steps.push(NextStep {
            command: "selfsign build".into(),
            why: "produce and sign desktop installers".into(),
        });
    } else {
        next_steps.push(NextStep {
            command: "selfsign build --skip-build".into(),
            why: "sign the installers already on disk".into(),
        });
        next_steps.push(NextStep {
            command: "selfsign release --tag v0.1.0 --dry-run".into(),
            why: "prepare GitHub Release assets".into(),
        });
    }
    next_steps.push(NextStep {
        command: "selfsign".into(),
        why: "open the TUI Guided setup if you prefer prompts".into(),
    });

    ScanReport {
        root,
        projects,
        installers,
        suggested: SuggestedConfig {
            project_name,
            tauri_root,
            windows,
            macos,
            linux,
            android,
            ios,
            notes,
        },
        next_steps,
        has_selfsign,
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
