use std::path::PathBuf;
use std::sync::mpsc;

use iced::widget::{operation, scrollable};
use iced::Task;

use cratty_core::{PaneId, Workspace, WorkspaceId};

use crate::message::Message;
use crate::pane::Pane;
use crate::strip_view::strip_scroll_id;
use crate::style::{CELL_H, CELL_W};
use crate::term_backend::TermBackend;
use crate::terminal;
use crate::{Cratty, PendingBackend};

impl Cratty {
    fn spawn_backend(&mut self, pane_id: PaneId, cwd: Option<PathBuf>) {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let (shell, args) = terminal::default_shell();
            let result = TermBackend::new(shell, args, cwd, 80, 24, CELL_W, CELL_H);
            let _ = tx.send(result);
        });
        self.pending_backends.push(PendingBackend { pane_id, rx });
    }

    pub fn create_workspace(&mut self, cwd: Option<PathBuf>) -> Task<Message> {
        let ws_id = self.id_gen.next_workspace();
        let pane_id = self.id_gen.next_pane();
        self.panes.insert(pane_id, Pane::new_loading(pane_id));
        let mut ws = Workspace::new(ws_id, format!("Terminal {}", ws_id.0));
        ws.strip.push(pane_id);
        self.workspaces.push(ws);
        self.active_ws = self.workspaces.len() - 1;
        self.spawn_backend(pane_id, cwd);
        scroll_to_strip_start()
    }

    pub fn add_pane_to_active_workspace(&mut self, cwd: Option<PathBuf>) -> Task<Message> {
        let ws = match self.workspaces.get_mut(self.active_ws) {
            Some(ws) => ws,
            None => return self.create_workspace(cwd),
        };
        let pane_id = self.id_gen.next_pane();
        self.panes.insert(pane_id, Pane::new_loading(pane_id));
        ws.strip.push(pane_id);
        let divider = 2.0;
        let x: f32 = (0..ws.strip.focus_idx)
            .map(|i| ws.strip.pane_width_at(i, self.viewport_w) + divider)
            .sum();
        self.spawn_backend(pane_id, cwd);
        operation::scroll_to(strip_scroll_id(), scrollable::AbsoluteOffset { x, y: 0.0 })
    }

    pub fn close_workspace(&mut self, idx: usize) -> Task<Message> {
        if idx >= self.workspaces.len() { return Task::none(); }
        let active_id = self.workspaces.get(self.active_ws).map(|w| w.id);
        let ws = self.workspaces.remove(idx);
        for pane_id in &ws.strip.panes { self.panes.remove(pane_id); }
        self.ws_colors.remove(&ws.id);
        if self.workspaces.is_empty() {
            self.active_ws = 0;
            return Task::none();
        }
        self.active_ws = active_id
            .and_then(|id| self.workspaces.iter().position(|w| w.id == id))
            .unwrap_or(self.active_ws.min(self.workspaces.len() - 1));
        Task::none()
    }

    pub fn remove_pane(&mut self, pid: PaneId) -> Task<Message> {
        self.panes.remove(&pid);
        let active_id = self.workspaces.get(self.active_ws).map(|w| w.id);
        for ws in &mut self.workspaces { ws.strip.remove(pid); }
        let empty_ids: Vec<WorkspaceId> = self.workspaces.iter()
            .filter(|ws| ws.is_empty()).map(|ws| ws.id).collect();
        for id in &empty_ids { self.ws_colors.remove(id); }
        self.workspaces.retain(|ws| !ws.is_empty());
        self.active_ws = active_id
            .and_then(|id| self.workspaces.iter().position(|w| w.id == id))
            .unwrap_or(self.active_ws.min(self.workspaces.len().saturating_sub(1)));
        Task::none()
    }
}

fn scroll_to_strip_start() -> Task<Message> {
    operation::scroll_to(strip_scroll_id(), scrollable::AbsoluteOffset { x: 0.0, y: 0.0 })
}
