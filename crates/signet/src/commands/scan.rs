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

    /// Overwrite project name / app_root / framework from scan when using --apply
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
        // Fill omitted fields only (zero-friction apply).
        let mut filled = Vec::new();
        if cfg.project.framework.trim().is_empty() {
            cfg.project.framework = s.framework.clone();
            filled.push("framework");
        }
        if cfg.project.app_root.trim().is_empty() {
            cfg.project.app_root = s.app_root.clone();
            filled.push("app_root");
        }
        cfg.write(&config_path)?;
        console::blank();
        if filled.is_empty() {
            console::ok_line(
                "updated platforms in signet.toml (use --force to also replace name/app_root/framework)",
            );
        } else {
            console::ok_line(&format!(
                "updated platforms + filled {} (use --force to replace existing fields)",
                filled.join(", ")
            ));
        }
    } else if config_path.exists() && force {
        let mut cfg = Config::load(&config_path)?;
        cfg.project.name = s.project_name.clone();
        cfg.project.app_root = s.app_root.clone();
        cfg.project.framework = s.framework.clone();
        cfg.platforms.windows = s.windows;
        cfg.platforms.macos = s.macos;
        cfg.platforms.linux = s.linux;
        cfg.write(&config_path)?;
        console::blank();
        console::ok_line("rewrote project + platforms in signet.toml");
    } else {
        let mut cfg = Config::example(&s.project_name, &s.app_root);
        cfg.project.framework = s.framework.clone();
        cfg.platforms.windows = s.windows;
        cfg.platforms.macos = s.macos;
        cfg.platforms.linux = s.linux;

        // Multiple detected apps → draft [[targets]] for monorepo roots.
        if report.projects.len() > 1 {
            use crate::config::Target;
            use crate::scan::framework_id_for_kind;
            cfg.targets = report
                .projects
                .iter()
                .take(8)
                .enumerate()
                .map(|(i, p)| {
                    let fw = framework_id_for_kind(p.kind).to_string();
                    let root_rel = p
                        .path
                        .strip_prefix(root)
                        .map(|r| {
                            let s = r.to_string_lossy().replace('\\', "/");
                            if s.is_empty() {
                                ".".into()
                            } else {
                                s
                            }
                        })
                        .unwrap_or_else(|_| ".".into());
                    let id = p
                        .name
                        .clone()
                        .filter(|n| !n.is_empty())
                        .unwrap_or_else(|| format!("target{}", i + 1));
                    Target {
                        id,
                        framework: fw,
                        app_root: root_rel,
                        build_command: String::new(),
                    }
                })
                .collect();
            // Prefer suggested as [project] summary; keep targets list.
            console::note("multiple projects detected — wrote [[targets]] draft (edit ids/build_command as needed)");
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
