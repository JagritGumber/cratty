use alacritty_terminal::grid::Scroll;
use alacritty_terminal::term::TermMode;

use crate::Cratty;

impl Cratty {
    /// Scroll the focused pane's terminal viewport.
    /// Positive delta = up (back in history), negative = down.
    pub fn scroll_focused_term(&self, scroll: Scroll) {
        let ws = match self.workspaces.get(self.active_ws) {
            Some(ws) => ws,
            None => return,
        };
        let pid = match ws.strip.focused_pane() {
            Some(pid) => pid,
            None => return,
        };
        let backend = match self.panes.get(&pid).and_then(|p| p.backend.as_ref()) {
            Some(b) => b,
            None => return,
        };
        let mut t = backend.term.lock();
        if !t.mode().contains(TermMode::ALT_SCREEN) {
            t.scroll_display(scroll);
        }
    }
}
