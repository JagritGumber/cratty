use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::column_width::ColumnWidth;

/// Serializable snapshot of a single terminal pane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneLayout {
    pub width: ColumnWidth,
    pub cwd: Option<PathBuf>,
}

/// Serializable snapshot of one workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLayout {
    pub name: String,
    #[serde(default = "default_true")]
    pub auto_named: bool,
    #[serde(default)]
    pub root: Option<PathBuf>,
    pub panes: Vec<PaneLayout>,
    pub focus_idx: usize,
}

fn default_true() -> bool { true }

/// Serializable snapshot of the full window layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowLayout {
    pub workspaces: Vec<WorkspaceLayout>,
    pub active_ws: usize,
}

impl WindowLayout {
    pub fn layout_path() -> PathBuf {
        crate::config::AppConfig::config_dir().join("layout.json")
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::layout_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn load() -> Option<Self> {
        let path = Self::layout_path();
        let data = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&data).ok()
    }
}
