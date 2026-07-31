use std::fs;
use std::path::PathBuf;

use clap::Args as ClapArgs;

use crate::config::{Config, CONFIG_FILE_NAME, SECRETS_DIR_NAME};
use crate::scan::{print_human, scan_repository, ScanReport};

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

    /// Overwrite project name / tauri_root from scan when using --apply
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

    if config_path.exists() && !force {
        let mut cfg = Config::load(&config_path)?;
        cfg.platforms.windows = s.windows;
        cfg.platforms.macos = s.macos;
        cfg.platforms.linux = s.linux;
        cfg.write(&config_path)?;
        console::blank();
        console::ok_line(&format!(
            "updated platforms in signet.toml (use --force to also replace name/tauri_root)"
        ));
    } else if config_path.exists() && force {
        let mut cfg = Config::load(&config_path)?;
        cfg.project.name = s.project_name.clone();
        cfg.project.tauri_root = s.tauri_root.clone();
        cfg.platforms.windows = s.windows;
        cfg.platforms.macos = s.macos;
        cfg.platforms.linux = s.linux;
        cfg.write(&config_path)?;
        console::blank();
        console::ok_line("rewrote project + platforms in signet.toml");
    } else {
        let mut cfg = Config::example(&s.project_name, &s.tauri_root);
        cfg.platforms.windows = s.windows;
        cfg.platforms.macos = s.macos;
        cfg.platforms.linux = s.linux;
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
    console::numbered(1, "signet identity create", "create a local code-signing identity");
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
