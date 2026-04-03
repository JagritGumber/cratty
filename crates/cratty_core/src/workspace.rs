use crate::id::{PaneId, WorkspaceId};
use crate::paper_strip::PaperStrip;

/// A named workspace containing a paper strip of terminal panes.
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub strip: PaperStrip,
}

impl Workspace {
    pub fn new(id: WorkspaceId, name: String) -> Self {
        Self {
            id,
            name,
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
