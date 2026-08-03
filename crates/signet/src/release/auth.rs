//! GitHub release authentication readiness (doctor / release / guided).

use std::process::Command;

use which::which;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GithubAuthKind {
    GhLoggedIn,
    GhInstalledNotLoggedIn,
    TokenEnv {
        var: &'static str,
    },
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubAuthReport {
    pub kind: GithubAuthKind,
}

impl GithubAuthReport {
    pub fn ready(&self) -> bool {
        matches!(
            self.kind,
            GithubAuthKind::GhLoggedIn | GithubAuthKind::TokenEnv { .. }
        )
    }

    pub fn summary_line(&self) -> String {
        match self.kind {
            GithubAuthKind::GhLoggedIn => "ready (gh logged in)".into(),
            GithubAuthKind::GhInstalledNotLoggedIn => {
                "NOT READY (gh installed, not logged in)".into()
            }
            GithubAuthKind::TokenEnv { var } => format!("ready ({var} set)"),
            GithubAuthKind::Missing => "NOT READY (no gh login and no GH_TOKEN)".into(),
        }
    }

    pub fn doctor_detail(&self) -> String {
        match self.kind {
            GithubAuthKind::GhLoggedIn => "gh CLI logged in (`gh auth status` ok)".into(),
            GithubAuthKind::GhInstalledNotLoggedIn => {
                "gh found but not logged in — run `gh auth login` (or set GH_TOKEN)".into()
            }
            GithubAuthKind::TokenEnv { var } => format!("{var} set"),
            GithubAuthKind::Missing => {
                "missing — install `gh` + `gh auth login`, or set GH_TOKEN (see docs/release.md)".into()
            }
        }
    }

    /// Numbered setup steps for humans (doctor footer / release bail / guided).
    pub fn setup_guide(&self) -> String {
        match self.kind {
            GithubAuthKind::GhLoggedIn | GithubAuthKind::TokenEnv { .. } => {
                "GitHub auth is ready for `signet release`.".into()
            }
            GithubAuthKind::GhInstalledNotLoggedIn => {
                "GitHub auth setup:\n\
                   1. Run `gh auth login` and complete the browser/device flow\n\
                   2. Confirm with `gh auth status`\n\
                   3. Or set GH_TOKEN / GITHUB_TOKEN (classic PAT with `repo` scope)\n\
                 See docs/release.md#auth"
                    .into()
            }
            GithubAuthKind::Missing => {
                "GitHub auth setup:\n\
                   1. Install GitHub CLI: https://cli.github.com/  (or winget/brew/apt)\n\
                   2. Run `gh auth login` and complete the browser/device flow\n\
                   3. Confirm with `gh auth status`\n\
                   4. Or set GH_TOKEN / GITHUB_TOKEN (classic PAT with `repo` scope; fine-grained needs Contents+Metadata write)\n\
                 See docs/release.md#auth"
                    .into()
            }
        }
    }

    pub fn preflight_error(&self) -> String {
        format!(
            "cannot publish: {}\n\n{}",
            self.summary_line(),
            self.setup_guide()
        )
    }
}

/// Assess whether this environment can publish a GitHub Release.
pub fn assess_github_auth() -> GithubAuthReport {
    if let Some(var) = token_env_var() {
        return GithubAuthReport {
            kind: GithubAuthKind::TokenEnv { var },
        };
    }
    if which("gh").is_ok() {
        if gh_auth_logged_in() {
            GithubAuthReport {
                kind: GithubAuthKind::GhLoggedIn,
            }
        } else {
            GithubAuthReport {
                kind: GithubAuthKind::GhInstalledNotLoggedIn,
            }
        }
    } else {
        GithubAuthReport {
            kind: GithubAuthKind::Missing,
        }
    }
}

fn token_env_var() -> Option<&'static str> {
    for key in ["GH_TOKEN", "GITHUB_TOKEN"] {
        if let Ok(v) = std::env::var(key) {
            if !v.trim().is_empty() {
                return Some(key);
            }
        }
    }
    None
}

fn gh_auth_logged_in() -> bool {
    Command::new("gh")
        .args(["auth", "status"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_kinds() {
        assert!(GithubAuthReport {
            kind: GithubAuthKind::GhLoggedIn
        }
        .ready());
        assert!(GithubAuthReport {
            kind: GithubAuthKind::TokenEnv { var: "GH_TOKEN" }
        }
        .ready());
        assert!(!GithubAuthReport {
            kind: GithubAuthKind::GhInstalledNotLoggedIn
        }
        .ready());
        assert!(!GithubAuthReport {
            kind: GithubAuthKind::Missing
        }
        .ready());
    }

    #[test]
    fn missing_guide_mentions_install_and_token() {
        let r = GithubAuthReport {
            kind: GithubAuthKind::Missing,
        };
        let g = r.setup_guide();
        assert!(g.contains("cli.github.com"), "{g}");
        assert!(g.contains("gh auth login"), "{g}");
        assert!(g.contains("GH_TOKEN"), "{g}");
        assert!(g.contains("docs/release.md"), "{g}");
    }

    #[test]
    fn not_logged_in_guide_skips_install() {
        let r = GithubAuthReport {
            kind: GithubAuthKind::GhInstalledNotLoggedIn,
        };
        let g = r.setup_guide();
        assert!(g.contains("gh auth login"), "{g}");
        assert!(!g.contains("cli.github.com"), "{g}");
    }

    #[test]
    fn preflight_includes_summary_and_guide() {
        let r = GithubAuthReport {
            kind: GithubAuthKind::Missing,
        };
        let e = r.preflight_error();
        assert!(e.contains("NOT READY"), "{e}");
        assert!(e.contains("1."), "{e}");
    }
}
