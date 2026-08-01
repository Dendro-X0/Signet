use std::fs;
use std::path::{Path, PathBuf};

use clap::Args as ClapArgs;

use crate::config::{resolve_config_path, Config, CONFIG_FILE_NAME, SECRETS_DIR_NAME};

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Application / release name written into signet.toml
    #[arg(long, default_value = "my-app")]
    pub name: String,

    /// Path to Tauri app root (directory containing src-tauri), relative to cwd
    #[arg(long, default_value = ".")]
    pub tauri_root: String,

    /// Framework adapter id (tauri, electron, flutter, …)
    #[arg(long, default_value = "tauri")]
    pub framework: String,

    /// Optional build argv (required for some hybrid / iOS adapters)
    #[arg(long, default_value = "")]
    pub build_command: String,

    /// Directory to write config into (default: current directory)
    #[arg(long, default_value = ".")]
    pub path: PathBuf,

    /// Overwrite existing signet.toml
    #[arg(long)]
    pub force: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let root = args.path.canonicalize().unwrap_or(args.path.clone());
    let config_path = root.join(CONFIG_FILE_NAME);

    if config_path.exists() && !args.force {
        anyhow::bail!(
            "{} already exists (pass --force to overwrite)",
            config_path.display()
        );
    }

    let mut config = Config::example(&args.name, &args.tauri_root);
    config.project.framework = args.framework.trim().to_string();
    if config.project.framework.is_empty() {
        config.project.framework = "tauri".into();
    }
    config.project.build_command = args.build_command;
    config.write(&config_path)?;

    let secrets_dir = root.join(SECRETS_DIR_NAME);
    fs::create_dir_all(&secrets_dir)?;
    ensure_gitignore(&root)?;
    write_secrets_readme(&secrets_dir)?;

    println!("wrote {}", config_path.display());
    println!("created {}/ (gitignored; keep keys here)", SECRETS_DIR_NAME);
    println!("next: signet identity create");
    Ok(())
}

fn ensure_gitignore(root: &Path) -> anyhow::Result<()> {
    let gi = root.join(".gitignore");
    let marker = SECRETS_DIR_NAME;
    let line = format!("{marker}/");

    if gi.exists() {
        let contents = fs::read_to_string(&gi)?;
        if contents.lines().any(|l| l.trim() == line.trim_end_matches('/') || l.trim() == line) {
            return Ok(());
        }
        let mut next = contents;
        if !next.ends_with('\n') && !next.is_empty() {
            next.push('\n');
        }
        next.push_str(&line);
        next.push('\n');
        fs::write(&gi, next)?;
    } else {
        fs::write(&gi, format!("# signet secrets — never commit\n{line}\n"))?;
    }
    Ok(())
}

fn write_secrets_readme(secrets_dir: &Path) -> anyhow::Result<()> {
    let readme = secrets_dir.join("README.md");
    if readme.exists() {
        return Ok(());
    }
    fs::write(
        readme,
        "# signet secrets\n\n\
         This directory holds private key material. Do not commit it.\n\n\
         See `docs/secrets-layout.md` in the signet repository for the layout.\n\
         Run `signet identity create` to generate a local signing cert + key.\n",
    )?;
    Ok(())
}

/// Used by other commands / TUI when locating config.
#[allow(dead_code)]
pub fn load_config(explicit: Option<&Path>) -> anyhow::Result<(PathBuf, Config)> {
    let path = resolve_config_path(explicit);
    let config = Config::load(&path).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok((path, config))
}
