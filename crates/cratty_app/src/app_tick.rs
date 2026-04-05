use cratty_core::PaneId;

use crate::style::{CELL_H, CELL_W};
use crate::Cratty;

impl Cratty {
    pub fn poll_pending_backends(&mut self) {
        self.pending_backends.retain(|pending| {
            match pending.rx.try_recv() {
                Ok(Ok(backend)) => {
                    if let Some(pane) = self.panes.get_mut(&pending.pane_id) {
                        pane.backend = Some(backend);
                        pane.title = "Terminal".into();
                    }
                    false // remove from pending
                }
                Ok(Err(e)) => {
                    tracing::error!("Failed to create terminal: {e}");
                    self.panes.remove(&pending.pane_id);
                    false
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => true, // keep waiting
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.panes.remove(&pending.pane_id);
                    false
                }
            }
        });
    }

    pub fn apply_pending_resizes(&mut self) {
        for pane in self.panes.values_mut() {
            if let Some(backend) = &mut pane.backend {
                backend.apply_pending_resize(CELL_W, CELL_H);
            }
        }
    }

    pub fn process_terminal_events(&mut self) {
        let pane_ids: Vec<PaneId> = self.panes.keys().copied().collect();
        for pid in pane_ids {
            let events = match self.panes.get(&pid).and_then(|p| p.backend.as_ref()) {
                Some(backend) => backend.drain_events(),
                None => continue,
            };
            for ev in events {
                match ev {
                    alacritty_terminal::event::Event::Title(title) => {
                        if let Some(pane) = self.panes.get_mut(&pid) {
                            pane.title = title;
                        }
                    }
                    alacritty_terminal::event::Event::Exit => {
                        self.panes.remove(&pid);
                        for ws in &mut self.workspaces { ws.strip.remove(pid); }
                    }
                    _ => {}
                }
            }
        }
        self.workspaces.retain(|ws| !ws.is_empty());
        if self.active_ws >= self.workspaces.len() && !self.workspaces.is_empty() {
            self.active_ws = self.workspaces.len() - 1;
        }
    }
}
