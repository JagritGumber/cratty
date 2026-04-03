use crate::id::PaneId;
use crate::view_offset::ViewOffset;

/// How wide each pane column is in the strip.
#[derive(Debug, Clone, Copy)]
pub enum ColumnWidth {
    /// Fraction of the visible strip width (e.g. 0.8 = 80%).
    Proportion(f32),
    /// Fixed pixel width.
    Fixed(f32),
}

impl Default for ColumnWidth {
    fn default() -> Self {
        Self::Proportion(1.0)
    }
}

/// A horizontal scrollable strip of terminal panes.
pub struct PaperStrip {
    pub panes: Vec<PaneId>,
    pub focus_idx: usize,
    pub viewport: ViewOffset,
    pub default_width: ColumnWidth,
}

impl PaperStrip {
    pub fn new() -> Self {
        Self {
            panes: Vec::new(),
            focus_idx: 0,
            viewport: ViewOffset::default(),
            default_width: ColumnWidth::default(),
        }
    }

    pub fn focused_pane(&self) -> Option<PaneId> {
        self.panes.get(self.focus_idx).copied()
    }

    pub fn push(&mut self, pane: PaneId) {
        self.panes.push(pane);
        self.focus_idx = self.panes.len() - 1;
    }

    pub fn remove(&mut self, pane: PaneId) -> bool {
        if let Some(idx) = self.panes.iter().position(|&p| p == pane) {
            self.panes.remove(idx);
            if self.focus_idx >= self.panes.len() && !self.panes.is_empty() {
                self.focus_idx = self.panes.len() - 1;
            }
            true
        } else {
            false
        }
    }

    pub fn focus_left(&mut self) -> bool {
        if self.focus_idx > 0 {
            self.focus_idx -= 1;
            true
        } else {
            false
        }
    }

    pub fn focus_right(&mut self) -> bool {
        if self.focus_idx + 1 < self.panes.len() {
            self.focus_idx += 1;
            true
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.panes.is_empty()
    }
}

impl Default for PaperStrip {
    fn default() -> Self {
        Self::new()
    }
}
