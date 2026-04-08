use std::path::PathBuf;

use cratty_core::PaneId;

use crate::term_backend::TermBackend;

/// A terminal pane -- wraps our PTY backend with display metadata.
pub struct Pane {
    pub id: PaneId,
    pub backend: Option<TermBackend>,
    pub title: String,
    pub custom_title: Option<String>,
    pub cwd: Option<PathBuf>,
}

impl Pane {
    pub fn new(id: PaneId, backend: TermBackend) -> Self {
        Self {
            id, backend: Some(backend),
            title: "Terminal".into(), custom_title: None, cwd: None,
        }
    }

    pub fn new_loading(id: PaneId) -> Self {
        Self {
            id, backend: None,
            title: "Loading...".into(), custom_title: None, cwd: None,
        }
    }

    pub fn display_title(&self) -> &str {
        self.custom_title.as_deref().unwrap_or(&self.title)
    }
}
