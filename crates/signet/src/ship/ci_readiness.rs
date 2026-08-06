//! CI secrets readiness for doctor / ship --plan (slice S2).

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use crate::android::keystore_paths;
use crate::config::Config;
use crate::release::assess_github_auth;
use crate::ship::coverage::mobile_commitment;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CiGap {
    pub id: &'static str,
    pub detail: String,
    pub next: String,
}

#[derive(Debug, Clone)]
pub struct CiReadinessReport {
    pub required: Vec<&'static str>,
    pub remote_present: BTreeSet<String>,
    pub remote_checked: bool,
    pub gaps: Vec<CiGap>,
}

impl CiReadinessReport {
    pub fn summary_line(&self) -> String {
        if self.gaps.is_empty() {
            format!("CI secrets: ready ({} required)", self.required.len())
        } else {
            let ids: Vec<&str> = self.gaps.iter().map(|g| g.id).collect();
            format!("CI secrets: missing — [{}]", ids.join(", "))
        }
    }

    pub fn print_human(&self) {
        use crate::ui::console;
        console::section("CI readiness");
        console::kv(14, "secrets", &self.summary_line());
        console::kv(14, "required", &self.required.join(", "));
        if self.remote_checked {
            console::kv(
                14,
                "gh-secrets",
                &format!("{} listed on remote", self.remote_present.len()),
            );
        } else {
            console::kv(14, "gh-secrets", "(not checked — GitHub auth not ready)");
        }
        if self.gaps.is_empty() {
            console::kv(14, "gaps", "(none)");
        } else {
            for g in &self.gaps {
                console::note(&format!("{} — {} ({})", g.id, g.detail, g.next));
            }
        }
    }
}

/// Secret names required for declared platforms / mobile commitment.
pub fn required_ci_secrets(config: &Config) -> Vec<&'static str> {
    let (android, _ios) = mobile_commitment(config);
    let desktop = config.platforms.windows || config.platforms.macos || config.platforms.linux;
    let mut names = Vec::new();
    if desktop {
        names.push("SIGNET_IDENTITY_BUNDLE_BASE64");
        if config.trust.checksum_signing.minisign {
            names.push("SIGNET_SUMS_KEY_BASE64");
        }
    }
    if android || config.platforms.android {
        names.push("SIGNET_ANDROID_KEYSTORE_BASE64");
        names.push("SIGNET_ANDROID_META_BASE64");
        names.push("SIGNET_ANDROID_STORE_PASS");
    }
    names
}

pub fn assess_ci_readiness(root: &Path, config: &Config) -> CiReadinessReport {
    let required = required_ci_secrets(config);
    let secrets = config.secrets_path(root);
    let mut gaps = Vec::new();

    let auth = assess_github_auth();
    let (remote_checked, remote_present) = if auth.ready() {
        (true, list_gh_secrets())
    } else {
        gaps.push(CiGap {
            id: "gap.github.auth",
            detail: auth.summary_line(),
            next: "gh auth login or set GH_TOKEN — see `signet doctor`".into(),
        });
        (false, BTreeSet::new())
    };

    let (android, ios) = mobile_commitment(config);
    if android || config.platforms.android {
        let ks = keystore_paths(&secrets);
        if !ks.keystore.is_file() {
            gaps.push(CiGap {
                id: "gap.android.keystore_local",
                detail: "local Android keystore missing".into(),
                next: "signet android keystore create".into(),
            });
        }
        let pass_ok = std::env::var("SIGNET_ANDROID_STORE_PASS")
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        let remote_ks = remote_present.contains("SIGNET_ANDROID_KEYSTORE_BASE64");
        let remote_meta = remote_present.contains("SIGNET_ANDROID_META_BASE64");
        let remote_pass = remote_present.contains("SIGNET_ANDROID_STORE_PASS");
        if remote_checked && (!remote_ks || !remote_meta || !remote_pass) {
            gaps.push(CiGap {
                id: "gap.android.ci_secrets",
                detail: format!(
                    "github secrets keystore={} meta={} store_pass={}",
                    if remote_ks { "ok" } else { "missing" },
                    if remote_meta { "ok" } else { "missing" },
                    if remote_pass { "ok" } else { "missing" }
                ),
                next: "signet ship secrets --push --apply".into(),
            });
        } else if !remote_checked && ks.keystore.is_file() && !pass_ok {
            gaps.push(CiGap {
                id: "gap.android.ci_secrets",
                detail: "local keystore present; store pass / remote secrets not verified".into(),
                next: "export SIGNET_ANDROID_STORE_PASS=… then signet ship secrets --push --apply"
                    .into(),
            });
        }
    }

    let desktop = config.platforms.windows || config.platforms.macos || config.platforms.linux;
    if desktop {
        let active = secrets.join("identity/active");
        if !active.is_file() {
            gaps.push(CiGap {
                id: "gap.desktop.identity_local",
                detail: "no active Signet identity".into(),
                next: "signet identity create".into(),
            });
        } else if remote_checked && !remote_present.contains("SIGNET_IDENTITY_BUNDLE_BASE64") {
            gaps.push(CiGap {
                id: "gap.desktop.ci_identity",
                detail: "SIGNET_IDENTITY_BUNDLE_BASE64 not on GitHub".into(),
                next: "signet ship secrets --push --apply".into(),
            });
        }
    }

    if ios || config.platforms.ios {
        gaps.push(CiGap {
            id: "gap.ios.codesign",
            detail: "Apple development team / provisioning is external to Signet".into(),
            next: "configure Xcode signing on the macOS runner (or skip iOS explicitly)".into(),
        });
    }

    CiReadinessReport {
        required,
        remote_present,
        remote_checked,
        gaps,
    }
}

fn list_gh_secrets() -> BTreeSet<String> {
    let output = Command::new("gh")
        .args(["secret", "list"])
        .output();
    let Ok(out) = output else {
        return BTreeSet::new();
    };
    if !out.status.success() {
        return BTreeSet::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| {
            let name = line.split_whitespace().next()?;
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Target};

    #[test]
    fn required_desktop_and_android() {
        let mut cfg = Config::example("app", ".");
        cfg.platforms.android = true;
        let names = required_ci_secrets(&cfg);
        assert!(names.contains(&"SIGNET_IDENTITY_BUNDLE_BASE64"));
        assert!(names.contains(&"SIGNET_ANDROID_KEYSTORE_BASE64"));
    }

    #[test]
    fn ios_gap_on_expo() {
        let mut cfg = Config::example("app", ".");
        cfg.platforms.windows = false;
        cfg.platforms.macos = false;
        cfg.platforms.linux = false;
        cfg.targets.push(Target {
            id: "m".into(),
            framework: "expo".into(),
            app_root: "m".into(),
            build_command: String::new(),
        });
        let report = assess_ci_readiness(Path::new("."), &cfg);
        assert!(
            report.gaps.iter().any(|g| g.id == "gap.ios.codesign"),
            "{:?}",
            report.gaps
        );
        assert!(report.gaps.iter().any(|g| g.id == "gap.android.keystore_local"));
    }
}
