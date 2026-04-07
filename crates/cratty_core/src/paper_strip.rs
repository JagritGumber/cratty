use crate::column_width::{self, ColumnWidth};
use crate::id::PaneId;
use crate::view_offset::ViewOffset;

pub struct PaperStrip {
    pub panes: Vec<PaneId>,
    pub widths: Vec<ColumnWidth>,
    pub focus_idx: usize,
    pub viewport: ViewOffset,
    pub default_width: ColumnWidth,
    pub preset_widths: Vec<ColumnWidth>,
    maximized: Option<(usize, ColumnWidth)>,
}

impl PaperStrip {
    pub fn new() -> Self {
        let presets = column_width::default_presets();
        Self {
            panes: Vec::new(), widths: Vec::new(), focus_idx: 0,
            viewport: ViewOffset::default(),
            default_width: presets[1], preset_widths: presets, maximized: None,
        }
    }

    pub fn focused_pane(&self) -> Option<PaneId> { self.panes.get(self.focus_idx).copied() }

    pub fn push(&mut self, pane: PaneId) {
        self.panes.push(pane);
        self.widths.push(self.default_width);
        self.focus_idx = self.panes.len() - 1;
    }

    pub fn remove(&mut self, pane: PaneId) -> bool {
        if let Some(idx) = self.panes.iter().position(|&p| p == pane) {
            self.panes.remove(idx);
            self.widths.remove(idx);
            if self.panes.is_empty() {
                self.focus_idx = 0;
            } else if idx < self.focus_idx {
                self.focus_idx -= 1;
            } else if self.focus_idx >= self.panes.len() {
                self.focus_idx = self.panes.len() - 1;
            }
            true
        } else {
            false
        }
    }

    pub fn focus_left(&mut self) -> bool {
        if self.focus_idx > 0 { self.focus_idx -= 1; true } else { false }
    }

    pub fn focus_right(&mut self) -> bool {
        if self.focus_idx + 1 < self.panes.len() { self.focus_idx += 1; true } else { false }
    }

    pub fn is_empty(&self) -> bool { self.panes.is_empty() }

    pub fn pane_width_at(&self, i: usize, vw: f32) -> f32 {
        self.widths.get(i).copied().unwrap_or(self.default_width).resolve(vw)
    }

    pub fn cycle_preset_width(&mut self) {
        if self.preset_widths.is_empty() || self.widths.is_empty() { return; }
        let current = self.widths[self.focus_idx];
        let pos = self.preset_widths.iter().position(|w| current.same_as(*w));
        let next = match pos {
            Some(i) => (i + 1) % self.preset_widths.len(),
            None => 0,
        };
        self.widths[self.focus_idx] = self.preset_widths[next];
        self.maximized = None;
    }

    pub fn adjust_focused_width(&mut self, delta: f32) {
        if self.widths.is_empty() { return; }
        self.widths[self.focus_idx] = self.widths[self.focus_idx].adjust(delta);
        self.maximized = None;
    }

    pub fn toggle_maximize(&mut self) {
        if self.widths.is_empty() { return; }
        if let Some((idx, prev)) = self.maximized.take() {
            if idx == self.focus_idx { self.widths[idx] = prev; return; }
        }
        let prev = self.widths[self.focus_idx];
        self.widths[self.focus_idx] = ColumnWidth::Proportion(1.0);
        self.maximized = Some((self.focus_idx, prev));
    }
}

impl Default for PaperStrip { fn default() -> Self { Self::new() } }
