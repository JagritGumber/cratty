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
    pub saved_scroll_x: f32,
    maximized: Option<(usize, ColumnWidth)>,
}

impl PaperStrip {
    pub fn new() -> Self {
        let presets = column_width::default_presets();
        Self {
            panes: Vec::new(), widths: Vec::new(), focus_idx: 0,
            viewport: ViewOffset::default(),
            default_width: presets[1], preset_widths: presets,
            saved_scroll_x: 0.0, maximized: None,
        }
    }

    pub fn focused_pane(&self) -> Option<PaneId> { self.panes.get(self.focus_idx).copied() }

    pub fn push(&mut self, pane: PaneId) {
        self.panes.push(pane);
        self.widths.push(self.default_width);
        self.focus_idx = self.panes.len() - 1;
    }

    pub fn remove(&mut self, pane: PaneId) -> bool {
        let Some(idx) = self.panes.iter().position(|&p| p == pane) else { return false };
        self.panes.remove(idx);
        self.widths.remove(idx);
        if self.panes.is_empty() { self.focus_idx = 0; }
        else if idx < self.focus_idx { self.focus_idx -= 1; }
        else if self.focus_idx >= self.panes.len() { self.focus_idx = self.panes.len() - 1; }
        true
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
        let cur = self.widths[self.focus_idx];
        let pos = self.preset_widths.iter().position(|w| cur.same_as(*w));
        let next = pos.map_or(0, |i| (i + 1) % self.preset_widths.len());
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

    pub fn swap_left(&mut self) -> bool { self.swap_adjacent(false) }
    pub fn swap_right(&mut self) -> bool { self.swap_adjacent(true) }

    fn swap_adjacent(&mut self, right: bool) -> bool {
        let target = if right { self.focus_idx + 1 } else { self.focus_idx.wrapping_sub(1) };
        if target >= self.panes.len() { return false; }
        self.panes.swap(self.focus_idx, target);
        self.widths.swap(self.focus_idx, target);
        self.focus_idx = target;
        self.maximized = None;
        true
    }
}

impl Default for PaperStrip { fn default() -> Self { Self::new() } }
