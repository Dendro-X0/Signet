use std::env;
use std::process::Command;

use clap::Args as ClapArgs;
use which::which;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Print JSON summary instead of human text
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Severity {
    /// Missing → non-zero exit
    Required,
    /// Missing → reported only
    Optional,
}

#[derive(Debug)]
struct Check {
    name: String,
    ok: bool,
    severity: Severity,
    detail: String,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let checks = gather_checks();
    let hard_failures = checks
        .iter()
        .filter(|c| !c.ok && c.severity == Severity::Required)
        .count();

    if args.json {
        print_json(&checks);
    } else {
        print_human(&checks);
    }

    if hard_failures > 0 {
        anyhow::bail!("{hard_failures} required check(s) failed");
    }
    Ok(())
}

fn gather_checks() -> Vec<Check> {
    let mut checks = Vec::new();

    checks.push(tool_check("rustc", &["--version"], Severity::Required));
    checks.push(tool_check("cargo", &["--version"], Severity::Required));

    let node = which("node").is_ok();
    checks.push(Check {
        name: "node".into(),
        ok: node,
        severity: Severity::Optional,
        detail: if node {
            version_of("node", &["--version"])
        } else {
            "not found (needed for most Tauri frontends)".into()
        },
    });

    let npm = which("npm").is_ok();
    checks.push(Check {
        name: "npm".into(),
        ok: npm,
        severity: Severity::Optional,
        detail: if npm {
            version_of("npm", &["--version"])
        } else {
            "not found".into()
        },
    });

    let tauri_cli = which("cargo-tauri").is_ok() || which("tauri").is_ok();
    checks.push(Check {
        name: "tauri-cli".into(),
        ok: tauri_cli,
        severity: Severity::Optional,
        detail: if tauri_cli {
            "found (cargo-tauri or tauri on PATH)".into()
        } else {
            "not found — install with: cargo install tauri-cli".into()
        },
    });

    let has_modern = std::path::Path::new("signet.toml").exists();
    let has_legacy = std::path::Path::new("selfsign.toml").exists();
    let has_config = has_modern || has_legacy;
    checks.push(Check {
        name: "signet.toml".into(),
        ok: has_config,
        severity: Severity::Optional,
        detail: if has_modern {
            "present in cwd".into()
        } else if has_legacy {
            "legacy selfsign.toml in cwd — prefer signet.toml".into()
        } else {
            "missing in cwd — run `signet init`".into()
        },
    });

    let identity_ok = std::path::Path::new(".signet/identity/active").exists()
        || std::path::Path::new(".selfsign/identity/active").exists();
    checks.push(Check {
        name: "identity".into(),
        ok: identity_ok,
        severity: Severity::Optional,
        detail: if identity_ok {
            "active identity present under .signet/ (or legacy .selfsign/)".into()
        } else {
            "no active identity — run `signet identity create`".into()
        },
    });

    checks.push(trust_tier_check(has_config, identity_ok));

    let gh = which("gh").is_ok();
    let token = std::env::var("GH_TOKEN")
        .or_else(|_| std::env::var("GITHUB_TOKEN"))
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    checks.push(Check {
        name: "github-auth".into(),
        ok: gh || token,
        severity: Severity::Optional,
        detail: if gh {
            "gh CLI found".into()
        } else if token {
            "GH_TOKEN/GITHUB_TOKEN set".into()
        } else {
            "missing — install `gh` or set GH_TOKEN for `signet release`".into()
        },
    });

    checks.extend(platform_checks());
    checks
}

fn trust_tier_check(has_config: bool, identity_ok: bool) -> Check {
    use crate::config::{resolve_config_path, Config};
    use crate::trust_tier::{resolve_primary_tier, TierHints};

    let root = std::path::Path::new(".");
    let hints = TierHints {
        has_active_identity: identity_ok,
        has_sha256sums: root.join("SHA256SUMS").is_file(),
        has_sums_signature: root.join("SHA256SUMS.minisig").is_file()
            || root.join("SHA256SUMS.asc").is_file(),
    };

    let (tier, source) = if has_config {
        match Config::load(resolve_config_path(None)) {
            Ok(cfg) => {
                let declared = cfg
                    .trust
                    .declared_tier
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .is_some();
                let tier = resolve_primary_tier(&cfg, hints);
                let source = if declared { "declared" } else { "inferred" };
                (tier, source)
            }
            Err(_) => (
                resolve_primary_tier(&Config::default(), hints),
                "inferred (config unreadable)",
            ),
        }
    } else {
        (
            resolve_primary_tier(&Config::default(), hints),
            "inferred (no config)",
        )
    };

    // Informational only — self_signed_host never fails the doctor run.
    Check {
        name: "trust-tier".into(),
        ok: true,
        severity: Severity::Optional,
        detail: format!(
            "{tier} ({source}) — integrity label, not OS reputation; see docs/trust-model.md"
        ),
    }
}

fn platform_checks() -> Vec<Check> {
    let mut checks = Vec::new();
    let os = env::consts::OS;

    checks.push(Check {
        name: "host-os".into(),
        ok: true,
        severity: Severity::Required,
        detail: format!("{os} ({})", env::consts::ARCH),
    });

    match os {
        "windows" => {
            let signtool = crate::sign::find_signtool();
            checks.push(Check {
                name: "signtool".into(),
                ok: signtool.is_some(),
                severity: Severity::Optional,
                detail: match signtool {
                    Some(p) => format!("found {}", p.display()),
                    None => {
                        "not found (Windows SDK Signing Tools; needed for `signet build`)"
                            .into()
                    }
                },
            });
            let openssl = crate::sign::find_openssl();
            checks.push(Check {
                name: "openssl".into(),
                ok: openssl.is_some(),
                severity: Severity::Optional,
                detail: match openssl {
                    Some(_) => "found (PFX export for signtool)".into(),
                    None => "not found (needed to export PFX from PEM identity)".into(),
                },
            });
        }
        "macos" => {
            let codesign = which("codesign").is_ok();
            checks.push(Check {
                name: "codesign".into(),
                ok: codesign,
                severity: Severity::Optional,
                detail: if codesign {
                    version_of("codesign", &["version"])
                } else {
                    "not found".into()
                },
            });
            let security = which("security").is_ok();
            checks.push(Check {
                name: "security".into(),
                ok: security,
                severity: Severity::Optional,
                detail: if security {
                    "found".into()
                } else {
                    "not found".into()
                },
            });
            let openssl = crate::sign::find_openssl().is_some();
            checks.push(Check {
                name: "openssl".into(),
                ok: openssl,
                severity: Severity::Optional,
                detail: if openssl {
                    "found (PFX export for keychain import)".into()
                } else {
                    "not found".into()
                },
            });
        }
        "linux" => {
            let openssl = crate::sign::find_openssl().is_some();
            checks.push(Check {
                name: "openssl".into(),
                ok: openssl,
                severity: Severity::Optional,
                detail: if openssl {
                    version_of("openssl", &["version"])
                } else {
                    "not found (detached signatures + checksums for Linux bundles)".into()
                },
            });
        }
        other => {
            checks.push(Check {
                name: "platform-tools".into(),
                ok: false,
                severity: Severity::Optional,
                detail: format!("unrecognized host OS for doctor hints: {other}"),
            });
        }
    }

    checks
}

fn tool_check(name: &str, version_args: &[&str], severity: Severity) -> Check {
    let ok = which(name).is_ok();
    Check {
        name: name.into(),
        ok,
        severity,
        detail: if ok {
            version_of(name, version_args)
        } else {
            "not found on PATH".into()
        },
    }
}

fn version_of(bin: &str, args: &[&str]) -> String {
    Command::new(bin)
        .args(args)
        .output()
        .ok()
        .and_then(|o| {
            let mut s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() {
                s = String::from_utf8_lossy(&o.stderr).trim().to_string();
            }
            if s.is_empty() {
                None
            } else {
                Some(s.lines().next().unwrap_or(&s).to_string())
            }
        })
        .unwrap_or_else(|| "found".into())
}

fn print_human(checks: &[Check]) {
    use crate::ui::console;

    console::banner("doctor");
    console::section("host checks");
    for c in checks {
        let ok = c.ok;
        let sev = match c.severity {
            Severity::Required => "required",
            Severity::Optional => "optional",
        };
        console::status(14, &c.name, ok, &format!("({sev}) {}", c.detail));
    }
    console::blank();
    console::note("`signet release --dry-run` lists assets; publish needs gh or GH_TOKEN.");
    console::blank();
}

fn print_json(checks: &[Check]) {
    println!("{{");
    println!("  \"checks\": [");
    for (i, c) in checks.iter().enumerate() {
        let comma = if i + 1 == checks.len() { "" } else { "," };
        let detail = c.detail.replace('\\', "\\\\").replace('"', "\\\"");
        let sev = match c.severity {
            Severity::Required => "required",
            Severity::Optional => "optional",
        };
        println!(
            "    {{\"name\": \"{}\", \"ok\": {}, \"severity\": \"{sev}\", \"detail\": \"{detail}\"}}{comma}",
            c.name, c.ok
        );
    }
    println!("  ]");
    println!("}}");
}
