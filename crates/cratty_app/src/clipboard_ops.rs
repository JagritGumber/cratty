use std::borrow::Cow;

use alacritty_terminal::event::Notify;
use alacritty_terminal::event_loop::Notifier;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Point};
use alacritty_terminal::term::TermMode;

use crate::{clipboard, Cratty};

impl Cratty {
    /// Copy visible screen content from the focused terminal to clipboard.
    pub fn copy_focused_term(&self) {
        let backend = match self.focused_backend() {
            Some(b) => b,
            None => return,
        };
        let t = backend.term.lock();
        let start = Point::new(t.topmost_line(), Column(0));
        let end = Point::new(t.bottommost_line(), t.last_column());
        let text = t.bounds_to_string(start, end);
        drop(t);
        if let Err(e) = clipboard::set_text(&text) {
            tracing::warn!("{e}");
        }
    }

    /// Paste clipboard text into the focused terminal's PTY.
    pub fn paste_to_focused_term(&self) {
        let text = match clipboard::get_text() {
            Ok(t) if !t.is_empty() => t,
            Ok(_) => return,
            Err(e) => { tracing::warn!("{e}"); return; }
        };
        let backend = match self.focused_backend() {
            Some(b) => b,
            None => return,
        };
        let bracketed = {
            let t = backend.term.lock();
            t.mode().contains(TermMode::BRACKETED_PASTE)
        };
        let notifier = Notifier(backend.sender.clone());
        if bracketed {
            notifier.notify(Cow::Borrowed(b"\x1b[200~" as &[u8]));
        }
        notifier.notify(Cow::Owned(text.into_bytes()));
        if bracketed {
            notifier.notify(Cow::Borrowed(b"\x1b[201~" as &[u8]));
        }
    }

    fn focused_backend(&self) -> Option<&crate::term_backend::TermBackend> {
        let ws = self.workspaces.get(self.active_ws)?;
        let pid = ws.strip.focused_pane()?;
        self.panes.get(&pid)?.terminal()
    }
}
