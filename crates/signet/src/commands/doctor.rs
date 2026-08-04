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
    checks.push(sums_minisign_key_check(has_config));
    checks.push(gpg_check_if_configured(has_config));
    checks.push(electron_npm_check(has_config));
    checks.push(hybrid_tool_check(has_config));
    checks.extend(android_tool_checks(has_config));
    checks.extend(ios_tool_checks(has_config));

    let auth = crate::release::assess_github_auth();
    checks.push(Check {
        name: "github-auth".into(),
        ok: auth.ready(),
        severity: Severity::Optional,
        detail: auth.doctor_detail(),
    });

    checks.push(ship_coverage_check(has_config));

    checks.extend(platform_checks());
    checks
}

fn ship_coverage_check(has_config: bool) -> Check {
    if !has_config {
        return Check {
            name: "ship-coverage".into(),
            ok: true,
            severity: Severity::Optional,
            detail: "no signet.toml — skip platform commitment check".into(),
        };
    }
    match crate::project::ProjectCtx::load(None) {
        Ok(ctx) => {
            let report = crate::ship::assess_coverage(&ctx.root, &ctx.config);
            if report.has_gap() {
                Check {
                    name: "ship-coverage".into(),
                    ok: false,
                    severity: Severity::Optional,
                    detail: format!(
                        "{} — gap [{}]; `signet ship --plan` for detail",
                        report.summary_line(),
                        report.gap.join(", ")
                    ),
                }
            } else {
                Check {
                    name: "ship-coverage".into(),
                    ok: true,
                    severity: Severity::Optional,
                    detail: report.summary_line(),
                }
            }
        }
        Err(e) => Check {
            name: "ship-coverage".into(),
            ok: true,
            severity: Severity::Optional,
            detail: format!("skipped ({e})"),
        },
    }
}

fn effective_framework() -> Option<String> {
    use crate::project::ProjectCtx;
    ProjectCtx::load(None).ok().map(|ctx| ctx.framework())
}

fn ios_tool_checks(has_config: bool) -> Vec<Check> {
    let is_ios = has_config
        && effective_framework()
            .map(|fw| fw.eq_ignore_ascii_case("ios"))
            .unwrap_or(false);

    let mut checks = Vec::new();
    let on_mac = env::consts::OS == "macos";

    let codesign_ok = which("codesign").is_ok();
    checks.push(Check {
        name: "ios-codesign".into(),
        ok: codesign_ok || !is_ios || !on_mac,
        severity: Severity::Optional,
        detail: if codesign_ok {
            "found (Apple code signing tool)".into()
        } else if is_ios && on_mac {
            "not found — install Xcode command-line tools".into()
        } else if is_ios {
            "iOS device signing tools are macOS-only; IPA zip packaging still works".into()
        } else {
            "not required unless framework = ios".into()
        },
    });

    let xcodebuild_ok = which("xcodebuild").is_ok();
    checks.push(Check {
        name: "xcodebuild".into(),
        ok: xcodebuild_ok || !is_ios || !on_mac,
        severity: Severity::Optional,
        detail: if xcodebuild_ok {
            "found".into()
        } else if is_ios && on_mac {
            "not found — set build_command after installing Xcode, or use --skip-build".into()
        } else {
            "not required unless building iOS on this host".into()
        },
    });

    checks
}

fn android_tool_checks(has_config: bool) -> Vec<Check> {
    use crate::android::{find_apksigner, find_keytool, keystore_paths};
    use crate::config::{resolve_config_path, Config};

    let is_android = has_config
        && effective_framework()
            .map(|fw| fw.eq_ignore_ascii_case("android"))
            .unwrap_or(false);

    let mut checks = Vec::new();

    let keytool_ok = find_keytool().is_some();
    checks.push(Check {
        name: "keytool".into(),
        ok: keytool_ok || !is_android,
        severity: Severity::Optional,
        detail: if keytool_ok {
            "found (JDK — android keystore create/import)".into()
        } else if is_android {
            "not found — install a JDK for `signet android keystore`".into()
        } else {
            "not required unless using Android helpers".into()
        },
    });

    let apksigner_ok = find_apksigner().is_some();
    checks.push(Check {
        name: "apksigner".into(),
        ok: apksigner_ok || !is_android,
        severity: Severity::Optional,
        detail: if apksigner_ok {
            "found (Android SDK build-tools)".into()
        } else if is_android {
            "not found — install Android SDK build-tools (jarsigner fallback may work)".into()
        } else {
            "not required unless signing APKs".into()
        },
    });

    let root = std::path::Path::new(".");
    let secrets = if has_config {
        Config::load(resolve_config_path(None))
            .map(|c| c.secrets_path(root))
            .unwrap_or_else(|_| root.join(".signet"))
    } else {
        root.join(".signet")
    };
    let ks = keystore_paths(&secrets);
    checks.push(Check {
        name: "android-keystore".into(),
        ok: ks.exists() || !is_android,
        severity: Severity::Optional,
        detail: if ks.exists() {
            format!("present at {}", ks.dir.display())
        } else if is_android {
            "missing — run `signet android keystore create`".into()
        } else {
            "not required unless framework = android".into()
        },
    });

    checks
}

fn electron_npm_check(has_config: bool) -> Check {
    let is_electron = has_config
        && effective_framework()
            .map(|fw| fw.eq_ignore_ascii_case("electron"))
            .unwrap_or(false);

    if !is_electron {
        return Check {
            name: "electron-npm".into(),
            ok: true,
            severity: Severity::Optional,
            detail: "not required (framework is not electron)".into(),
        };
    }

    let ok = which("npm").is_ok() || which("npx").is_ok();
    Check {
        name: "electron-npm".into(),
        ok,
        severity: Severity::Optional,
        detail: if ok {
            "found (needed for Electron `signet build` unless --skip-build)".into()
        } else {
            "not found — install Node.js/npm or use --skip-build / custom build_command".into()
        },
    }
}

fn hybrid_tool_check(has_config: bool) -> Check {
    let fw = if has_config {
        effective_framework()
            .map(|fw| fw.to_ascii_lowercase())
            .unwrap_or_default()
    } else {
        String::new()
    };

    match fw.as_str() {
        "flutter" => {
            let ok = which("flutter").is_ok();
            Check {
                name: "flutter-sdk".into(),
                ok,
                severity: Severity::Optional,
                detail: if ok {
                    "found (needed for `signet build` unless --skip-build)".into()
                } else {
                    "not found — install Flutter SDK or pass --skip-build".into()
                },
            }
        }
        "react-native" | "rn" | "expo" | "capacitor" => {
            let ok = which("npm").is_ok() || which("npx").is_ok();
            Check {
                name: "hybrid-npm".into(),
                ok,
                severity: Severity::Optional,
                detail: if ok {
                    format!("found (framework={fw}; set build_command — see docs/frameworks.md)")
                } else {
                    "not found — install Node.js/npm or use --skip-build".into()
                },
            }
        }
        _ => Check {
            name: "hybrid-tools".into(),
            ok: true,
            severity: Severity::Optional,
            detail: "not required (framework is not flutter/rn/expo/capacitor)".into(),
        },
    }
}

fn sums_minisign_key_check(has_config: bool) -> Check {
    use crate::config::{resolve_config_path, Config};
    use crate::sign::SumsKeyPaths;

    let root = std::path::Path::new(".");
    let secrets = if has_config {
        Config::load(resolve_config_path(None))
            .map(|c| c.secrets_path(root))
            .unwrap_or_else(|_| root.join(".signet"))
    } else {
        root.join(".signet")
    };
    let paths = SumsKeyPaths::from_secrets_dir(&secrets);
    let ok = paths.exists();
    Check {
        name: "sums-minisign-key".into(),
        ok,
        severity: Severity::Optional,
        detail: if ok {
            format!("present at {}", paths.dir.display())
        } else {
            "missing — run `signet sums-key create` to sign SHA256SUMS".into()
        },
    }
}

fn gpg_check_if_configured(has_config: bool) -> Check {
    use crate::config::{resolve_config_path, Config};

    let gpg_wanted = has_config
        && Config::load(resolve_config_path(None))
            .map(|c| c.trust.checksum_signing.gpg)
            .unwrap_or(false);

    if !gpg_wanted {
        return Check {
            name: "gpg".into(),
            ok: true,
            severity: Severity::Optional,
            detail: "not required ([trust.checksum_signing].gpg = false)".into(),
        };
    }

    let ok = which("gpg").is_ok();
    Check {
        name: "gpg".into(),
        ok,
        severity: Severity::Optional,
        detail: if ok {
            version_of("gpg", &["--version"])
        } else {
            "not found — required because [trust.checksum_signing].gpg = true".into()
        },
    }
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
            checks.push(Check {
                name: "graduation-windows".into(),
                ok: true,
                severity: Severity::Optional,
                detail: "OV/Azure: `signet graduate ov-sign|azure-sign` — docs/graduation.md \
                         (not Signet self-signed identity)"
                    .into(),
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
            let notary = Command::new("xcrun")
                .args(["--find", "notarytool"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            checks.push(Check {
                name: "notarytool".into(),
                ok: notary,
                severity: Severity::Optional,
                detail: if notary {
                    "found (Apple notarization — `signet graduate notarize`)"
                        .into()
                } else {
                    "not found (needed for `signet graduate notarize`)".into()
                },
            });
            let stapler = Command::new("xcrun")
                .args(["--find", "stapler"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            checks.push(Check {
                name: "stapler".into(),
                ok: stapler,
                severity: Severity::Optional,
                detail: if stapler {
                    "found (`signet graduate staple`)".into()
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
    if let Some(_auth_check) = checks.iter().find(|c| c.name == "github-auth" && !c.ok) {
        console::section("github release auth");
        let auth = crate::release::assess_github_auth();
        for line in auth.setup_guide().lines() {
            console::note(line);
        }
        console::blank();
        let _ = crate::release::offer_open_auth_setup(&auth);
    }
    console::note("`signet release --dry-run` lists assets; live publish needs ready github-auth.");
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
