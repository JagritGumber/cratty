use crate::column_width::ColumnWidth;
use crate::paper_strip::PaperStrip;
use crate::view_offset::ease_out_quart;

#[derive(Debug, Clone, Copy)]
pub struct WidthAnim {
    pub from: f32,
    pub progress: f32,
}

impl PaperStrip {
    /// Rendered width, interpolated if an animation is in progress.
    pub fn pane_width_rendered(&self, i: usize, vw: f32) -> f32 {
        let target = self.pane_width_at(i, vw);
        match self.width_anims.get(i).and_then(|a| a.as_ref()) {
            Some(a) => a.from + (target - a.from) * ease_out_quart(a.progress),
            None => target,
        }
    }

    /// Advance all width animations. Returns true if any still animating.
    pub fn tick_width_anims(&mut self, dt: f32) -> bool {
        let mut going = false;
        for slot in &mut self.width_anims {
            if let Some(anim) = slot {
                anim.progress = (anim.progress + dt).min(1.0);
                if anim.progress >= 1.0 { *slot = None; } else { going = true; }
            }
        }
        going
    }

    fn start_anim(&mut self, idx: usize, from_px: f32) {
        if idx < self.width_anims.len() {
            self.width_anims[idx] = Some(WidthAnim { from: from_px, progress: 0.0 });
        }
    }

    pub fn cycle_preset_width(&mut self, vw: f32) {
        if self.preset_widths.is_empty() || self.widths.is_empty() { return; }
        let from_px = self.pane_width_rendered(self.focus_idx, vw);
        let cur = self.widths[self.focus_idx];
        let pos = self.preset_widths.iter().position(|w| cur.same_as(*w));
        let next = pos.map_or(0, |i| (i + 1) % self.preset_widths.len());
        self.widths[self.focus_idx] = self.preset_widths[next];
        self.maximized = None;
        self.start_anim(self.focus_idx, from_px);
    }

    pub fn adjust_focused_width(&mut self, delta: f32, vw: f32) {
        if self.widths.is_empty() { return; }
        let from_px = self.pane_width_rendered(self.focus_idx, vw);
        self.widths[self.focus_idx] = self.widths[self.focus_idx].adjust(delta);
        self.maximized = None;
        self.start_anim(self.focus_idx, from_px);
    }

    pub fn toggle_maximize(&mut self, vw: f32) {
        if self.widths.is_empty() { return; }
        let from_px = self.pane_width_rendered(self.focus_idx, vw);
        if let Some((idx, prev)) = self.maximized.take() {
            if idx == self.focus_idx {
                self.widths[idx] = prev;
                self.start_anim(idx, from_px);
                return;
            }
        }
        let prev = self.widths[self.focus_idx];
        self.widths[self.focus_idx] = ColumnWidth::Proportion(1.0);
        self.maximized = Some((self.focus_idx, prev));
        self.start_anim(self.focus_idx, from_px);
    }
}
