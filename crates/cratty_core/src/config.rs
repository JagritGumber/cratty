use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_font_family")]
    pub font_family: String,

    #[serde(default = "default_font_size")]
    pub font_size: f32,

    #[serde(default)]
    pub shell_profiles: Vec<ShellProfile>,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_scrollback")]
    pub scrollback_lines: u32,

    #[serde(default)]
    pub quake_mode: QuakeModeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellProfile {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: Vec<(String, String)>,
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuakeModeConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_quake_hotkey")]
    pub hotkey: String,
    #[serde(default = "default_quake_height")]
    pub height_percent: f32,
}

impl Default for QuakeModeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hotkey: default_quake_hotkey(),
            height_percent: default_quake_height(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            font_family: default_font_family(),
            font_size: default_font_size(),
            shell_profiles: default_shell_profiles(),
            theme: default_theme(),
            scrollback_lines: default_scrollback(),
            quake_mode: QuakeModeConfig::default(),
        }
    }
}

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

fn default_font_family() -> String {
    "JetBrains Mono".to_string()
}
fn default_font_size() -> f32 {
    14.0
}
fn default_theme() -> String {
    "default-dark".to_string()
}
fn default_scrollback() -> u32 {
    10_000
}
fn default_quake_hotkey() -> String {
    "F12".to_string()
}
fn default_quake_height() -> f32 {
    0.4
}
fn default_true() -> bool {
    true
}

fn default_shell_profiles() -> Vec<ShellProfile> {
    let mut profiles = Vec::new();

    #[cfg(windows)]
    {
        profiles.push(ShellProfile {
            name: "PowerShell".to_string(),
            command: "pwsh.exe".to_string(),
            args: vec![],
            env: vec![],
            working_directory: None,
        });
        profiles.push(ShellProfile {
            name: "Command Prompt".to_string(),
            command: "cmd.exe".to_string(),
            args: vec![],
            env: vec![],
            working_directory: None,
        });
    }

    #[cfg(not(windows))]
    {
        if let Ok(shell) = std::env::var("SHELL") {
            profiles.push(ShellProfile {
                name: "Default Shell".to_string(),
                command: shell,
                args: vec!["-l".to_string()],
                env: vec![],
                working_directory: None,
            });
        }
    }

    profiles
}
