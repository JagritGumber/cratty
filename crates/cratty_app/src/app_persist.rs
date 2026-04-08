use cratty_core::{PaneLayout, WindowLayout, WorkspaceLayout};

use crate::Cratty;

impl Cratty {
    /// Snapshot the current window state into a serializable layout.
    pub fn save_layout(&self) {
        let workspaces = self.workspaces.iter().map(|ws| {
            let panes = ws.strip.panes.iter().zip(ws.strip.widths.iter())
                .map(|(pid, width)| {
                    let cwd = self.panes.get(pid).and_then(|p| p.cwd.clone());
                    PaneLayout { width: *width, cwd }
                })
                .collect();
            WorkspaceLayout {
                name: ws.name.clone(),
                panes,
                focus_idx: ws.strip.focus_idx,
            }
        }).collect();

        let layout = WindowLayout {
            workspaces,
            active_ws: self.active_ws,
        };

        if let Err(e) = layout.save() {
            tracing::warn!("Failed to save layout: {e}");
        }
    }

    /// Restore workspaces from the saved layout file.
    pub fn restore_layout(&mut self) {
        let layout = match WindowLayout::load() {
            Some(l) if !l.workspaces.is_empty() => l,
            _ => return,
        };

        for ws_layout in &layout.workspaces {
            let ws_id = self.id_gen.next_workspace();
            let mut ws = cratty_core::Workspace::new(
                ws_id, ws_layout.name.clone(),
            );
            for pane_layout in &ws_layout.panes {
                let pane_id = self.id_gen.next_pane();
                self.panes.insert(
                    pane_id,
                    crate::pane::Pane::new_loading(pane_id),
                );
                ws.strip.push(pane_id);
                let idx = ws.strip.panes.len() - 1;
                ws.strip.widths[idx] = pane_layout.width;
                self.spawn_backend(
                    pane_id, pane_layout.cwd.clone(),
                );
            }
            ws.strip.focus_idx = ws_layout.focus_idx
                .min(ws.strip.panes.len().saturating_sub(1));
            self.workspaces.push(ws);
        }
        self.active_ws = layout.active_ws
            .min(self.workspaces.len().saturating_sub(1));
    }
}
