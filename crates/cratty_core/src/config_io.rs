use std::path::{Path, PathBuf};

use crate::config::AppConfig;

impl AppConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            Ok(serde_yaml::from_str(&contents)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("cratty")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.yaml")
    }
}
