use std::fs;
use std::path::PathBuf;

use clap::Args as ClapArgs;

use crate::config::{Config, CONFIG_FILE_NAME, SECRETS_DIR_NAME};
use crate::scan::{
    draft_targets, merge_platforms, print_human, scan_repository, ScanReport,
};

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Repository root to scan (default: current directory)
    #[arg(long, default_value = ".")]
    pub path: PathBuf,

    /// Print JSON report
    #[arg(long)]
    pub json: bool,

    /// Write / update signet.toml from the suggested desktop signing config
    #[arg(long)]
    pub apply: bool,

    /// Overwrite project name / app_root / framework / platforms / targets from scan
    #[arg(long)]
    pub force: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let report = scan_repository(&args.path)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human(&report);
    }

    if args.apply {
        apply_suggestion(&report, args.force)?;
    }

    Ok(())
}

fn apply_suggestion(report: &ScanReport, force: bool) -> anyhow::Result<()> {
    use crate::ui::console;

    let root = &report.root;
    let config_path = root.join(CONFIG_FILE_NAME);
    let s = &report.suggested;
    let drafted = draft_targets(root, &report.projects);

    if config_path.exists() {
        let mut cfg = Config::load(&config_path)?;
        let mut parts: Vec<String> = Vec::new();

        if force {
            cfg.project.name = s.project_name.clone();
            cfg.project.app_root = s.app_root.clone();
            cfg.project.framework = s.framework.clone();
            parts.push("project fields".into());
        } else {
            let mut filled = Vec::new();
            if cfg.project.framework.trim().is_empty() {
                cfg.project.framework = s.framework.clone();
                filled.push("framework");
            }
            if cfg.project.app_root.trim().is_empty() {
                cfg.project.app_root = s.app_root.clone();
                filled.push("app_root");
            }
            if !filled.is_empty() {
                parts.push(format!("filled {}", filled.join(", ")));
            }
        }

        let before = cfg.platforms.clone();
        let kept_shipping_intent = !force
            && ((before.macos && !s.macos) || (before.linux && !s.linux) || (before.windows && !s.windows));
        cfg.platforms = merge_platforms(&before, s.windows, s.macos, s.linux, force);
        if force {
            parts.push("platforms (forced)".into());
        } else if cfg.platforms != before {
            parts.push("platforms (expanded only)".into());
        } else {
            parts.push("platforms unchanged".into());
        }

        if force {
            if !drafted.is_empty() {
                cfg.targets = drafted.clone();
                parts.push("[[targets]] replaced".into());
            }
        } else if cfg.targets.is_empty() && !drafted.is_empty() {
            cfg.targets = drafted.clone();
            parts.push("[[targets]] drafted".into());
        }

        cfg.write(&config_path)?;
        console::blank();
        if parts.is_empty() {
            console::ok_line("signet.toml unchanged (use --force to replace fields)");
        } else {
            console::ok_line(&format!("updated signet.toml — {}", parts.join("; ")));
        }
        if kept_shipping_intent
            && ((cfg.platforms.macos && !s.macos) || (cfg.platforms.linux && !s.linux))
        {
            console::note(
                "kept macos/linux=true (shipping intent); pass --force to adopt host-shaped platforms",
            );
        }
        if !force && cfg.targets.is_empty() && drafted.is_empty() && report.projects.len() > 1 {
            console::note(
                "multiple detections but fewer than 2 installable apps — no [[targets]] draft",
            );
        }
        if !force && !cfg.targets.is_empty() && !drafted.is_empty() && parts.iter().all(|p| !p.contains("[[targets]]"))
        {
            console::note("[[targets]] already set — left unchanged (pass --force to replace)");
        }
    } else {
        let mut cfg = Config::example(&s.project_name, &s.app_root);
        cfg.project.framework = s.framework.clone();
        cfg.platforms.windows = s.windows;
        cfg.platforms.macos = s.macos;
        cfg.platforms.linux = s.linux;
        if !drafted.is_empty() {
            cfg.targets = drafted;
            console::note(
                "multiple installable apps — wrote [[targets]] draft (edit ids/build_command as needed)",
            );
        }

        cfg.write(&config_path)?;

        let secrets = root.join(SECRETS_DIR_NAME);
        fs::create_dir_all(&secrets)?;
        ensure_gitignore(root)?;
        let readme = secrets.join("README.md");
        if !readme.exists() {
            fs::write(
                readme,
                "# signet secrets\n\nDo not commit private keys.\n",
            )?;
        }
        console::blank();
        console::ok_line("wrote signet.toml from scan suggestion (desktop platforms only)");
    }

    if s.android || s.ios {
        console::note(
            "android/ios detected but not stored in [platforms] — desktop self-sign only",
        );
    }
    if !report.has_identity {
        console::numbered(1, "signet identity create", "create a local code-signing identity");
    } else {
        console::note("identity already present — next: `signet build` / `signet trust`");
    }
    Ok(())
}

fn ensure_gitignore(root: &std::path::Path) -> anyhow::Result<()> {
    let gi = root.join(".gitignore");
    let line = format!("{SECRETS_DIR_NAME}/");
    if gi.exists() {
        let contents = fs::read_to_string(&gi)?;
        if contents.lines().any(|l| l.trim() == line || l.trim() == SECRETS_DIR_NAME) {
            return Ok(());
        }
        let mut next = contents;
        if !next.ends_with('\n') && !next.is_empty() {
            next.push('\n');
        }
        next.push_str(&line);
        next.push('\n');
        fs::write(gi, next)?;
    } else {
        fs::write(gi, format!("# signet secrets\n{line}\n"))?;
    }
    Ok(())
}
