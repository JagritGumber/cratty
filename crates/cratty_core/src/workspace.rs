use std::path::PathBuf;

use crate::id::{PaneId, WorkspaceId};
use crate::paper_strip::PaperStrip;

/// A named workspace containing a paper strip of panes.
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    /// True if the name was auto-generated; false if user-set.
    pub auto_named: bool,
    /// Project root folder. None for legacy workspaces.
    pub root: Option<PathBuf>,
    /// Shared replaceable preview pane for file opens.
    pub preview_pane: Option<PaneId>,
    pub strip: PaperStrip,
}

impl Workspace {
    pub fn new(id: WorkspaceId, name: String) -> Self {
        Self { id, name, auto_named: true, root: None, preview_pane: None, strip: PaperStrip::new() }
    }

    pub fn with_root(id: WorkspaceId, root: PathBuf) -> Self {
        let name = root.file_name().and_then(|s| s.to_str()).map(String::from)
            .unwrap_or_else(|| format!("Workspace {}", id.0));
        Self { id, name, auto_named: false, root: Some(root), preview_pane: None, strip: PaperStrip::new() }
    }

    pub fn focused_pane(&self) -> Option<PaneId> { self.strip.focused_pane() }
    pub fn is_empty(&self) -> bool { self.strip.is_empty() }
}
