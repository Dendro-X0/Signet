//! Resolve project root, config, and secrets paths from `signet.toml`.

use std::path::{Path, PathBuf};

use crate::config::{resolve_config_path, Config, ConfigError};

#[derive(Debug, Clone)]
pub struct ProjectCtx {
    #[allow(dead_code)] // useful for diagnostics / future relative resolution
    pub config_path: PathBuf,
    /// Directory containing `signet.toml`
    pub root: PathBuf,
    pub config: Config,
}

impl ProjectCtx {
    pub fn load(explicit_config: Option<&Path>) -> Result<Self, ConfigError> {
        let config_path = resolve_config_path(explicit_config);
        let config = Config::load(&config_path)?;
        let root = config_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        Ok(Self {
            config_path,
            root,
            config,
        })
    }

    pub fn secrets_dir(&self) -> PathBuf {
        self.config.secrets_path(&self.root)
    }

    pub fn identity_root(&self) -> PathBuf {
        self.secrets_dir().join("identity")
    }
}
