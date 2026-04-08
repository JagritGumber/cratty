use cratty_core::PaneId;

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
                    false
                }
                Ok(Err(e)) => {
                    tracing::error!("Failed to create terminal: {e}");
                    self.panes.remove(&pending.pane_id);
                    for ws in &mut self.workspaces {
                        ws.strip.remove(pending.pane_id);
                    }
                    false
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => true,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.panes.remove(&pending.pane_id);
                    for ws in &mut self.workspaces {
                        ws.strip.remove(pending.pane_id);
                    }
                    false
                }
            }
        });
    }

    pub fn tick_animations(&mut self) {
        if let Some(ws) = self.workspaces.get_mut(self.active_ws) {
            ws.strip.viewport.tick(0.06);
        }
    }

    pub fn apply_pending_resizes(&mut self) {
        let cell_w = self.metrics.cell_w;
        let cell_h = self.metrics.cell_h;
        for pane in self.panes.values_mut() {
            if let Some(backend) = &mut pane.backend {
                backend.apply_pending_resize(cell_w, cell_h);
            }
        }
    }

    pub fn process_terminal_events(&mut self) {
        let pane_ids: Vec<PaneId> = self.panes.keys().copied().collect();
        for pid in pane_ids {
            let events = {
                match self.panes.get(&pid).and_then(|p| p.backend.as_ref()) {
                    Some(backend) => backend.drain_events(),
                    None => continue,
                }
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
                        for ws in &mut self.workspaces {
                            ws.strip.remove(pid);
                        }
                    }
                    _ => {}
                }
            }
        }
        for ws in self.workspaces.iter().filter(|ws| ws.is_empty()) {
            self.ws_colors.remove(&ws.id);
        }
        self.workspaces.retain(|ws| !ws.is_empty());
        if self.active_ws >= self.workspaces.len()
            && !self.workspaces.is_empty()
        {
            self.active_ws = self.workspaces.len() - 1;
        }
    }
}
