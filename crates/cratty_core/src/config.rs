use serde::{Deserialize, Serialize};

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

impl ShellProfile {
    fn simple(name: &str, cmd: &str) -> Self {
        Self {
            name: name.into(), command: cmd.into(),
            args: vec![], env: vec![], working_directory: None,
        }
    }
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
        Self { enabled: true, hotkey: default_quake_hotkey(), height_percent: default_quake_height() }
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

fn default_font_family() -> String { "JetBrains Mono".into() }
fn default_font_size() -> f32 { 14.0 }
fn default_theme() -> String { "default-dark".into() }
fn default_scrollback() -> u32 { 10_000 }
fn default_quake_hotkey() -> String { "F12".into() }
fn default_quake_height() -> f32 { 0.4 }
fn default_true() -> bool { true }

fn default_shell_profiles() -> Vec<ShellProfile> {
    let mut profiles = Vec::new();
    #[cfg(windows)]
    {
        profiles.push(ShellProfile::simple("PowerShell", "pwsh.exe"));
        profiles.push(ShellProfile::simple("Command Prompt", "cmd.exe"));
    }
    #[cfg(not(windows))]
    {
        if let Ok(shell) = std::env::var("SHELL") {
            let mut p = ShellProfile::simple("Default Shell", &shell);
            p.args = vec!["-l".into()];
            profiles.push(p);
        }
    }
    profiles
}
