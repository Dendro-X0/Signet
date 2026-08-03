//! Guided flows — thin prompts over `commands::*` (no second behavior path).

use std::path::PathBuf;

use crate::artifact::select_adapter;
use crate::commands;
use crate::config::Config;
use crate::error::ExitCode;
use crate::identity;
use crate::project::ProjectCtx;
use crate::scan::scan_repository;
use crate::tui::framework_pick::{
    build_command_hint, index_of_framework, preferred_framework_from_projects,
    requires_build_command, FRAMEWORK_OPTIONS,
};
use crate::tui::prompts::{confirm, prompt_choice, prompt_line};
use crate::tui::status::ProjectStatus;
use crate::ui::console::{self, skip_line, step, step_active};

pub fn run_doctor() -> anyhow::Result<()> {
    console::banner("doctor");
    commands::doctor::run(commands::doctor::Args { json: false })
}

pub fn guided_scan() -> anyhow::Result<()> {
    console::banner("scan");
    commands::scan::run(commands::scan::Args {
        path: PathBuf::from("."),
        json: false,
        apply: false,
        force: false,
    })?;
    if confirm("apply suggested desktop config to signet.toml?", false)? {
        commands::scan::run(commands::scan::Args {
            path: PathBuf::from("."),
            json: false,
            apply: true,
            force: false,
        })?;
    }
    if confirm("continue into Guided setup?", true)? {
        console::blank();
        guided_setup_with(GuidedOpts { skip_scan: true })?;
    }
    Ok(())
}

pub fn run_trust() -> anyhow::Result<()> {
    console::banner("Prove · trust");
    commands::trust::run(commands::trust::Args {
        config: None,
        out: None,
    })
}

pub fn guided_verify() -> anyhow::Result<()> {
    console::banner("Check · verify");
    let code = commands::verify::run(commands::verify::Args {
        config: None,
        artifacts: vec![],
        sums: None,
        trust: None,
        require_sig: false,
        minisign_pub: None,
        json: false,
        fingerprint: None,
        fail_stale: false,
    });
    if code != ExitCode::Success {
        anyhow::bail!("verify exited with code {}", code as i32);
    }
    Ok(())
}

pub fn guided_inspect() -> anyhow::Result<()> {
    console::banner("Check · inspect");
    let files = sample_artifact_paths()?;
    if files.is_empty() {
        let path = prompt_line("artifact path to inspect", "")?;
        if path.is_empty() {
            anyhow::bail!("no artifact path given");
        }
        return run_inspect_files(&[PathBuf::from(path)]);
    }
    println!("  Found {} artifact(s) to inspect:", files.len());
    for f in &files {
        println!("    · {}", f.display());
    }
    if !confirm("inspect these files?", true)? {
        let path = prompt_line("or enter a path (empty = cancel)", "")?;
        if path.is_empty() {
            skip_line("cancelled");
            return Ok(());
        }
        return run_inspect_files(&[PathBuf::from(path)]);
    }
    run_inspect_files(&files)
}

fn run_inspect_files(files: &[PathBuf]) -> anyhow::Result<()> {
    let code = commands::inspect::run(commands::inspect::Args {
        files: files.to_vec(),
        json: false,
        strict: false,
    });
    if code != ExitCode::Success {
        anyhow::bail!("inspect exited with code {}", code as i32);
    }
    Ok(())
}

pub fn run_graduate_notes() -> anyhow::Result<()> {
    console::banner("Official Sign · graduate");
    commands::graduate::run(commands::graduate::Args {
        config: None,
        action: commands::graduate::Action::Notes,
    })
}

pub fn guided_init() -> anyhow::Result<()> {
    console::banner("init");
    let status = ProjectStatus::probe(".");
    if status.has_config {
        console::ok_line("signet.toml already exists");
        if !confirm("overwrite with --force?", false)? {
            skip_line("skipped");
            return Ok(());
        }
    }

    console::section("detect");
    let report = scan_repository(std::path::Path::new("."))?;
    let suggested = &report.suggested;
    if report.projects.is_empty() {
        console::muted("no project markers found — you can pick a framework manually");
    } else {
        for p in report.projects.iter().take(5) {
            let rel = p
                .path
                .strip_prefix(&report.root)
                .map(|r| {
                    let s = r.to_string_lossy().replace('\\', "/");
                    if s.is_empty() {
                        ".".into()
                    } else {
                        s
                    }
                })
                .unwrap_or_else(|_| p.path.display().to_string());
            console::bullet(&format!(
                "[{}] {} — {}",
                p.kind.as_str(),
                rel,
                p.detail
            ));
        }
        if report.projects.len() > 5 {
            console::muted(&format!(
                "{} more omitted — run Scan for full report",
                report.projects.len() - 5
            ));
        }
    }
    console::note(&format!(
        "suggested: framework={}  root={}  name={}",
        suggested.framework, suggested.app_root, suggested.project_name
    ));
    if suggested.framework == "cli" {
        console::note(
            "this looks like a CLI / tooling repo, not an installable desktop or mobile app",
        );
    }

    // Zero-friction path: accept scan suggestion with one confirm.
    if confirm("write suggested signet.toml (Enter = yes)?", true)? {
        let mut build_command = String::new();
        if crate::tui::framework_pick::requires_build_command(&suggested.framework) {
            console::note(crate::tui::framework_pick::build_command_hint(
                &suggested.framework,
            ));
            build_command = prompt_line("build_command (required)", "")?;
        }
        return commands::init::run(commands::init::Args {
            name: suggested.project_name.clone(),
            app_root: suggested.app_root.clone(),
            framework: suggested.framework.clone(),
            build_command,
            path: PathBuf::from("."),
            force: status.has_config,
        });
    }

    let default_name = status
        .app_name
        .as_deref()
        .unwrap_or(suggested.project_name.as_str());
    let name = prompt_line("app name", default_name)?;
    let app_root = prompt_line("app root", &suggested.app_root)?;
    let (framework, build_command) =
        prompt_framework_and_build_cmd(Some(suggested.framework.as_str()))?;
    commands::init::run(commands::init::Args {
        name,
        app_root,
        framework,
        build_command,
        path: PathBuf::from("."),
        force: status.has_config,
    })
}

fn prompt_framework_and_build_cmd(
    suggested: Option<&str>,
) -> anyhow::Result<(String, String)> {
    if let Some(fw) = suggested {
        let label = FRAMEWORK_OPTIONS
            .iter()
            .find(|(id, _)| *id == fw)
            .map(|(_, l)| *l)
            .unwrap_or(fw);
        console::note(&format!("detected framework: {label} ({fw})"));
        if confirm(&format!("use framework={fw}?"), true)? {
            return prompt_build_command_for(fw);
        }
    }
    let labels: Vec<&str> = FRAMEWORK_OPTIONS.iter().map(|(_, l)| *l).collect();
    let default_idx = suggested.map(index_of_framework).unwrap_or(0);
    let choice = prompt_choice("Framework adapter", &labels, default_idx)?;
    let framework = FRAMEWORK_OPTIONS[choice].0.to_string();
    prompt_build_command_for(&framework)
}

fn prompt_build_command_for(framework: &str) -> anyhow::Result<(String, String)> {
    let mut build_command = String::new();
    if requires_build_command(framework) {
        console::note(build_command_hint(framework));
        build_command = prompt_line(
            "build_command (required for this framework; empty only if you will --skip-build)",
            "",
        )?;
        if build_command.is_empty() {
            console::note("empty build_command — use Build → Sign only (--skip-build) later");
        }
    } else {
        console::note(build_command_hint(framework));
        if confirm("set a custom build_command?", false)? {
            build_command = prompt_line("build_command", "")?;
        }
    }
    Ok((framework.to_string(), build_command))
}

pub fn guided_identity() -> anyhow::Result<()> {
    console::banner("Sign · identity");
    let choice = prompt_choice(
        "Identity action",
        &[
            "Show active identity",
            "Create new identity",
            "List identities",
        ],
        if ProjectStatus::probe(".").has_identity {
            0
        } else {
            1
        },
    )?;

    match choice {
        0 => commands::identity::run(commands::identity::Args {
            config: None,
            action: Some(commands::identity::Action::Show(
                commands::identity::ShowArgs { name: None },
            )),
        }),
        1 => {
            let ctx = ProjectCtx::load(None).map_err(|e| {
                anyhow::anyhow!("{e}\nhint: run Init from the hub first")
            })?;
            let default_cn = format!("{} Code Signing", ctx.config.project.name);
            let name = prompt_line("identity name", "default")?;
            let cn = prompt_line("common name (CN)", &default_cn)?;
            let org = prompt_line("organization (optional)", "")?;
            let days: u32 = prompt_line("validity days", "825")?
                .parse()
                .unwrap_or(825);
            let exists = identity::load_named(&ctx.identity_root(), &name).is_ok();
            let force = exists && confirm("identity exists — overwrite?", false)?;

            commands::identity::run(commands::identity::Args {
                config: None,
                action: Some(commands::identity::Action::Create(
                    commands::identity::CreateArgs {
                        name,
                        cn: Some(cn),
                        org,
                        days,
                        force,
                    },
                )),
            })
        }
        _ => commands::identity::run(commands::identity::Args {
            config: None,
            action: Some(commands::identity::Action::List),
        }),
    }
}

pub fn guided_build() -> anyhow::Result<()> {
    console::banner("Sign · build");
    note_framework_mismatch(".")?;
    let mode = prompt_choice(
        "Build mode",
        &[
            "Full: framework build + sign",
            "Sign only (--skip-build)",
            "Build/discover only (--no-sign)",
        ],
        1,
    )?;
    let skip_build = mode == 1;
    let no_sign = mode == 2;
    let no_timestamp = if !no_sign && cfg!(windows) {
        !confirm("request Authenticode timestamp?", true)?
    } else {
        false
    };

    if !skip_build {
        if let Ok(ctx) = ProjectCtx::load(None) {
            let fw = ctx.framework();
            if requires_build_command(&fw) && ctx.config.project.build_command.trim().is_empty() {
                console::note(build_command_hint(&fw));
                anyhow::bail!(
                    "[{fw}] build_command is empty — set it in signet.toml or choose Sign only (--skip-build)"
                );
            }
        }
    }

    console::blank();
    commands::build::run(commands::build::Args {
        config: None,
        target: None,
        skip_build,
        no_sign,
        no_sums_sign: false,
        require_sums_sign: false,
        require_gpg: false,
        profile: "release".into(),
        no_timestamp,
        tauri_args: vec![],
        artifacts: vec![],
    })
}

pub fn guided_release() -> anyhow::Result<()> {
    console::banner("Prove · release");
    let default_tag = crate::version_detect::default_release_tag(std::path::Path::new("."));
    let tag = prompt_line("git tag", &default_tag)?;
    let repo = prompt_line("GitHub repo owner/name (empty = detect)", "")?;
    let mode = prompt_choice(
        "Publish mode",
        &[
            "Dry-run (list assets + notes, no upload)",
            "Publish to GitHub Releases",
        ],
        0,
    )?;
    let dry_run = mode == 0;
    let draft = if dry_run {
        false
    } else {
        confirm("create as draft?", false)?
    };
    let prerelease = if dry_run {
        false
    } else {
        confirm("mark as prerelease?", false)?
    };

    commands::release::run(commands::release::Args {
        config: None,
        tag: Some(tag),
        repo: if repo.is_empty() { None } else { Some(repo) },
        profile: "release".into(),
        dry_run,
        draft,
        prerelease,
        no_clobber: false,
        no_trust: false,
        no_sums_sign: false,
        require_sums_sign: false,
        require_gpg: false,
        title: None,
        artifacts: vec![],
    })
}

#[derive(Debug, Clone, Default)]
pub struct GuidedOpts {
    /// When true, do not offer the opening scan (caller already scanned).
    pub skip_scan: bool,
}

/// End-to-end first-release wizard (Sign → Prove → Check).
pub fn guided_setup() -> anyhow::Result<()> {
    guided_setup_with(GuidedOpts::default())
}

pub fn guided_setup_with(opts: GuidedOpts) -> anyhow::Result<()> {
    console::banner("guided setup · Sign → Prove → Check");
    println!("  Same engines as the CLI — prompts only collect arguments.");
    println!("  Cancel any step with Ctrl+C.");
    console::blank();

    let total = 8usize;
    let mut step_n = 1usize;

    // --- optional doctor ---
    step_active(step_n, total, "Doctor (optional)");
    if confirm("run doctor first?", true)? {
        run_doctor()?;
    } else {
        skip_line("skipped doctor");
    }
    step_n += 1;
    console::blank();

    // --- scan ---
    let mut suggested_fw: Option<String> = None;
    let mut suggested_root = ".".to_string();
    if !opts.skip_scan {
        step_active(step_n, total, "Scan · find app + framework");
        if confirm("scan the repo for projects/installers?", true)? {
            let report = scan_repository(std::path::Path::new("."))?;
            crate::scan::print_human(&report);
            suggested_fw = Some(report.suggested.framework.clone());
            suggested_root = report.suggested.app_root.clone();
            if let Some(ref fw) = suggested_fw {
                console::note(&format!("suggested framework: {fw}"));
            }
        } else {
            skip_line("skipped scan");
        }
    } else {
        step(step_n, total, "Scan · find app + framework", true);
        skip_line("scan already completed — continuing wizard");
        if let Ok(report) = scan_repository(std::path::Path::new(".")) {
            suggested_fw =
                preferred_framework_from_projects(&report.root, &report.projects)
                    .map(|s| s.to_string());
            suggested_root = report.suggested.app_root.clone();
        }
    }
    step_n += 1;
    console::blank();

    let mut status = ProjectStatus::probe(".");

    // --- init / framework ---
    step_active(step_n, total, "Init · config + framework");
    if !status.has_config {
        let name = prompt_line("app name", status.app_name.as_deref().unwrap_or("my-app"))?;
        let app_root = prompt_line("app root", &suggested_root)?;
        let (framework, build_command) =
            prompt_framework_and_build_cmd(suggested_fw.as_deref())?;
        commands::init::run(commands::init::Args {
            name,
            app_root,
            framework,
            build_command,
            path: PathBuf::from("."),
            force: false,
        })?;
    } else {
        console::ok_line("signet.toml present");
        if confirm("change framework / build_command now?", false)? {
            update_framework_in_config(suggested_fw.as_deref())?;
        } else if let Ok(ctx) = ProjectCtx::load(None) {
            let fw = ctx.framework();
            let src = if crate::config::framework_is_explicit(&ctx.config) {
                "config"
            } else {
                "scan"
            };
            console::note(&format!(
                "framework={fw} (from {src}) build_command={:?}",
                ctx.config.project.build_command
            ));
        }
    }
    step_n += 1;
    status = ProjectStatus::probe(".");
    console::blank();

    // --- Sign: identity ---
    step_active(step_n, total, "Sign · identity");
    if !status.has_identity {
        if confirm("create a signing identity now?", true)? {
            let ctx_name = ProjectCtx::load(None)
                .map(|c| c.config.project.name)
                .unwrap_or_else(|_| "my-app".into());
            let default_cn = format!("{ctx_name} Code Signing");
            let cn = prompt_line("common name (CN)", &default_cn)?;
            let org = prompt_line("organization (optional)", "")?;
            commands::identity::run(commands::identity::Args {
                config: None,
                action: Some(commands::identity::Action::Create(
                    commands::identity::CreateArgs {
                        name: "default".into(),
                        cn: Some(cn),
                        org,
                        days: 825,
                        force: false,
                    },
                )),
            })?;
        } else {
            skip_line("skipped identity — run it later from the hub");
            return Ok(());
        }
    } else {
        step(step_n, total, "Sign · identity", true);
        let _ = commands::identity::run(commands::identity::Args {
            config: None,
            action: Some(commands::identity::Action::Show(
                commands::identity::ShowArgs { name: None },
            )),
        });
    }
    step_n += 1;
    status = ProjectStatus::probe(".");
    console::blank();

    // --- Prove: trust ---
    step_active(step_n, total, "Prove · TRUST.md");
    if !status.has_trust || confirm("regenerate TRUST.md?", !status.has_trust)? {
        run_trust()?;
    } else {
        skip_line("kept existing TRUST.md");
    }
    step_n += 1;
    console::blank();

    // --- Sign: build ---
    step_active(step_n, total, "Sign · build / sign artifacts");
    if confirm("run build/sign now?", true)? {
        guided_build()?;
    } else {
        skip_line("skipped — run Build from the hub when artifacts are ready");
    }
    step_n += 1;
    console::blank();

    // --- Prove: release dry-run optional ---
    step_active(step_n, total, "Prove · release dry-run (optional)");
    if confirm("prepare a release dry-run?", false)? {
        let default_tag = crate::version_detect::default_release_tag(std::path::Path::new("."));
        let tag = prompt_line("git tag", &default_tag)?;
        let repo = prompt_line("GitHub repo owner/name (empty = detect)", "")?;
        commands::release::run(commands::release::Args {
            config: None,
            tag: Some(tag),
            repo: if repo.is_empty() { None } else { Some(repo) },
            profile: "release".into(),
            dry_run: true,
            draft: false,
            prerelease: false,
            no_clobber: false,
            no_trust: false,
            no_sums_sign: false,
            require_sums_sign: false,
            require_gpg: false,
            title: None,
            artifacts: vec![],
        })?;
        if confirm("publish for real now? (needs gh or GH_TOKEN)", false)? {
            guided_release()?;
        }
    } else {
        skip_line("skipped release");
    }
    step_n += 1;
    console::blank();

    // --- Check ---
    step_active(step_n, total, "Check · verify + inspect");
    let has_trust = std::path::Path::new("TRUST.md").is_file();
    let has_sums = std::path::Path::new("SHA256SUMS").is_file();
    if has_trust || has_sums {
        if confirm("run signet verify?", true)? {
            match guided_verify() {
                Ok(()) => console::ok_line("verify finished"),
                Err(e) => console::note(&format!("verify: {e}")),
            }
        }
    } else {
        skip_line("no TRUST.md / SHA256SUMS yet — skipped verify");
    }
    if confirm("run signet inspect on artifacts?", true)? {
        match guided_inspect() {
            Ok(()) => console::ok_line("inspect finished"),
            Err(e) => console::note(&format!("inspect: {e}")),
        }
    } else {
        skip_line("skipped inspect");
    }

    console::blank();
    console::ok_line("Guided setup finished (Sign → Prove → Check) for this host.");
    if let Ok(ctx) = ProjectCtx::load(None) {
        let cov = crate::ship::assess_coverage(&ctx.root, &ctx.config);
        console::note(&cov.summary_line());
        if cov.has_gap() {
            console::note(
                "Multi-OS gap remains — guided Check on this machine ≠ full [platforms] ship. Run `signet ship --plan`.",
            );
        }
    }
    console::note("Need OV / Azure / Apple notarize? → hub · Graduate notes  (or: signet graduate notes)");
    console::note("Prefer CLI flags in CI; use the hub when exploring.");
    Ok(())
}

fn update_framework_in_config(suggested: Option<&str>) -> anyhow::Result<()> {
    let path = PathBuf::from("signet.toml");
    let mut cfg = Config::load(&path)?;
    let (framework, build_command) = prompt_framework_and_build_cmd(suggested)?;
    cfg.project.framework = framework;
    cfg.project.build_command = build_command;
    cfg.write(&path)?;
    console::ok_line(&format!("updated {}", path.display()));
    Ok(())
}

/// Warn when an *explicit* signet.toml framework disagrees with scan (does not rewrite).
fn note_framework_mismatch(root: &str) -> anyhow::Result<()> {
    let Ok(ctx) = ProjectCtx::load(None) else {
        return Ok(());
    };
    if !crate::config::framework_is_explicit(&ctx.config) {
        return Ok(());
    }
    let Ok(report) = scan_repository(std::path::Path::new(root)) else {
        return Ok(());
    };
    let Some(suggested) =
        preferred_framework_from_projects(&report.root, &report.projects)
    else {
        return Ok(());
    };
    let configured = ctx.config.project.framework.as_str();
    if crate::version_detect::frameworks_equivalent(configured, suggested) {
        return Ok(());
    }
    console::note(&format!(
        "config framework={configured} but scan suggests {suggested} — \
         change via Init / edit signet.toml if build discovers the wrong artifacts"
    ));
    Ok(())
}

fn sample_artifact_paths() -> anyhow::Result<Vec<PathBuf>> {
    let ctx = match ProjectCtx::load(None) {
        Ok(c) => c,
        Err(_) => return Ok(vec![]),
    };
    let adapter = match select_adapter(&ctx.root, &ctx.config) {
        Ok(a) => a,
        Err(_) => return Ok(vec![]),
    };
    let arts = adapter.discover(&ctx, "release").unwrap_or_default();
    Ok(arts
        .into_iter()
        .filter(|a| a.path.exists())
        .map(|a| a.path)
        .take(5)
        .collect())
}

pub fn run_self_status() -> anyhow::Result<()> {
    commands::self_cmd::status_public()
}

pub fn run_self_update() -> anyhow::Result<()> {
    console::banner("Update Signet");
    let st = crate::self_manage::current_status();
    console::note(&st.detail);
    if !st.managed {
        console::note("Install via the README one-liner to enable self-update.");
        if !confirm("try update with --force anyway?", false)? {
            return Ok(());
        }
        return commands::self_cmd::run(commands::self_cmd::Args {
            command: commands::self_cmd::SelfCommand::Update {
                check: false,
                force: true,
            },
        });
    }
    if !confirm("download and install the latest release?", true)? {
        skip_line("skipped");
        return Ok(());
    }
    commands::self_cmd::update_default()
}

pub fn run_self_uninstall() -> anyhow::Result<()> {
    console::banner("Uninstall Signet");
    let st = crate::self_manage::current_status();
    if !st.managed {
        anyhow::bail!(
            "this copy is not installer-managed — cannot uninstall from TUI ({})",
            st.detail
        );
    }
    console::note(&format!("will remove {}", st.binary.display()));
    console::note("Project `.signet/` directories are not deleted.");
    if !confirm("uninstall the Signet CLI from this machine?", false)? {
        skip_line("cancelled");
        return Ok(());
    }
    commands::self_cmd::uninstall_confirmed()
}
