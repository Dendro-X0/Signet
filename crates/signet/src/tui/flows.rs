//! Guided flows — thin prompts over `commands::*` (no second behavior path).

use std::path::PathBuf;

use crate::commands;
use crate::identity;
use crate::project::ProjectCtx;
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
        guided_setup_with(GuidedOpts {
            skip_scan: true,
        })?;
    }
    Ok(())
}

pub fn run_trust() -> anyhow::Result<()> {
    console::banner("trust");
    commands::trust::run(commands::trust::Args {
        config: None,
        out: None,
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
    let name = prompt_line("app name", status.app_name.as_deref().unwrap_or("my-app"))?;
    let tauri_root = prompt_line("tauri_root (dir containing src-tauri)", ".")?;
    commands::init::run(commands::init::Args {
        name,
        tauri_root,
        path: PathBuf::from("."),
        force: status.has_config,
    })
}

pub fn guided_identity() -> anyhow::Result<()> {
    console::banner("identity");
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
    console::banner("build");
    let mode = prompt_choice(
        "Build mode",
        &[
            "Full: tauri build + sign",
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

    console::blank();
    commands::build::run(commands::build::Args {
        config: None,
        skip_build,
        no_sign,
        profile: "release".into(),
        no_timestamp,
        tauri_args: vec![],
        artifacts: vec![],
    })
}

pub fn guided_release() -> anyhow::Result<()> {
    console::banner("release");
    let tag = prompt_line("git tag", "v0.1.0")?;
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
        title: None,
        artifacts: vec![],
    })
}

#[derive(Debug, Clone, Default)]
pub struct GuidedOpts {
    /// When true, do not offer the opening scan (caller already scanned).
    pub skip_scan: bool,
}

/// End-to-end first-release wizard.
pub fn guided_setup() -> anyhow::Result<()> {
    guided_setup_with(GuidedOpts::default())
}

pub fn guided_setup_with(opts: GuidedOpts) -> anyhow::Result<()> {
    console::banner("guided setup");
    println!("  Same engines as the CLI — prompts only collect arguments.");
    println!("  Cancel any step with Ctrl+C.");
    console::blank();

    if !opts.skip_scan {
        if confirm("scan the repo for installers first?", true)? {
            commands::scan::run(commands::scan::Args {
                path: PathBuf::from("."),
                json: false,
                apply: false,
                force: false,
            })?;
            if !ProjectStatus::probe(".").has_config
                && confirm("apply suggested signet.toml from scan?", true)?
            {
                commands::scan::run(commands::scan::Args {
                    path: PathBuf::from("."),
                    json: false,
                    apply: true,
                    force: false,
                })?;
            }
            console::blank();
        }
    } else {
        skip_line("scan already completed — continuing wizard");
    }

    let mut status = ProjectStatus::probe(".");

    // 1. init
    if !status.has_config {
        step_active(1, 5, "init");
        guided_init()?;
    } else {
        step(1, 5, "init", true);
    }
    status = ProjectStatus::probe(".");

    // 2. identity
    if !status.has_identity {
        step_active(2, 5, "identity");
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
        step(2, 5, "identity", true);
        let _ = commands::identity::run(commands::identity::Args {
            config: None,
            action: Some(commands::identity::Action::Show(
                commands::identity::ShowArgs { name: None },
            )),
        });
    }
    status = ProjectStatus::probe(".");

    // 3. trust
    step_active(3, 5, "trust");
    if !status.has_trust || confirm("regenerate TRUST.md?", !status.has_trust)? {
        run_trust()?;
    } else {
        skip_line("kept existing TRUST.md");
    }

    // 4. build
    step_active(4, 5, "build");
    if confirm("run build/sign now?", false)? {
        guided_build()?;
    } else {
        skip_line("skipped — run Build from the hub when artifacts are ready");
    }

    // 5. release dry-run
    step_active(5, 5, "release");
    if confirm("prepare a release dry-run?", true)? {
        let tag = prompt_line("git tag", "v0.1.0")?;
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
            title: None,
            artifacts: vec![],
        })?;
        if confirm("publish for real now? (needs gh or GH_TOKEN)", false)? {
            guided_release()?;
        }
    } else {
        skip_line("skipped release");
    }

    console::blank();
    console::ok_line("Guided setup finished. Prefer CLI flags in CI; use the hub when exploring.");
    Ok(())
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
