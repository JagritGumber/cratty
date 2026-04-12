use crate::id::{PaneId, WorkspaceId};
use crate::paper_strip::PaperStrip;

/// A named workspace containing a paper strip of terminal panes.
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    /// True if the name was auto-generated (e.g. "Terminal 1"); false if user-set.
    /// When true, the sidebar prefers CWD basename for display.
    pub auto_named: bool,
    pub strip: PaperStrip,
}

impl Workspace {
    pub fn new(id: WorkspaceId, name: String) -> Self {
        Self {
            id, name, auto_named: true,
            strip: PaperStrip::new(),
        }
    }

    pub fn focused_pane(&self) -> Option<PaneId> {
        self.strip.focused_pane()
    }

    pub fn is_empty(&self) -> bool {
        self.strip.is_empty()
    }
}
