use std::path::PathBuf;
use std::sync::mpsc;
use iced::widget::{operation, scrollable};
use iced::Task;
use cratty_core::{PaneId, Workspace, WorkspaceId};
use crate::message::Message;
use crate::pane::Pane;
use crate::strip_view::strip_scroll_id;
use crate::term_backend::TermBackend;
use crate::terminal;
use crate::{Cratty, PendingBackend};

impl Cratty {
    pub fn spawn_backend(&mut self, pane_id: PaneId, cwd: Option<PathBuf>) {
        let (shell, args) = terminal::default_shell();
        self.spawn_process(pane_id, shell, args, cwd);
    }

    pub fn spawn_process(
        &mut self, pane_id: PaneId, command: String, args: Vec<String>, cwd: Option<PathBuf>,
    ) {
        let cell_w = self.metrics.cell_w;
        let cell_h = self.metrics.cell_h;
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(TermBackend::new(command, args, cwd, 80, 24, cell_w, cell_h));
        });
        self.pending_backends.retain(|pending| pending.pane_id != pane_id);
        self.pending_backends.push(PendingBackend { pane_id, rx });
    }

    fn install_workspace(&mut self, mut ws: Workspace, cwd: Option<PathBuf>) -> Task<Message> {
        let pane_id = self.id_gen.next_pane();
        self.panes.insert(pane_id, Pane::new_loading(pane_id));
        ws.strip.push(pane_id);
        self.workspaces.push(ws);
        self.active_ws = self.workspaces.len() - 1;
        self.spawn_backend(pane_id, cwd);
        if let Some(ws) = self.workspaces.get_mut(self.active_ws) {
            ws.strip.viewport = cratty_core::ViewOffset::Static(0.0);
        }
        scroll_to_strip_start()
    }

    pub fn create_workspace(&mut self, cwd: Option<PathBuf>) -> Task<Message> {
        let id = self.id_gen.next_workspace();
        self.install_workspace(Workspace::new(id, format!("Terminal {}", id.0)), cwd)
    }

    pub fn create_workspace_with_root(&mut self, root: PathBuf) -> Task<Message> {
        let id = self.id_gen.next_workspace();
        self.install_workspace(Workspace::with_root(id, root.clone()), Some(root))
    }

    pub fn add_pane_to_active_workspace(&mut self, cwd: Option<PathBuf>) -> Task<Message> {
        let ws = match self.workspaces.get_mut(self.active_ws) {
            Some(ws) => ws, None => return self.create_workspace(cwd),
        };
        let inherited_cwd = cwd.or_else(|| ws.root.clone()).or_else(|| {
            ws.focused_pane().and_then(|pid| self.panes.get(&pid)).and_then(|p| p.cwd.clone())
        });
        let pane_id = self.id_gen.next_pane();
        self.panes.insert(pane_id, Pane::new_loading(pane_id));
        let ws = self.workspaces.get_mut(self.active_ws).unwrap();
        ws.strip.push(pane_id);
        let vw = self.viewport_w;
        let x: f32 = (0..ws.strip.focus_idx).map(|i| ws.strip.pane_width_at(i, vw) + 2.0).sum();
        ws.strip.viewport = cratty_core::ViewOffset::Static(x);
        self.spawn_backend(pane_id, inherited_cwd);
        operation::scroll_to(strip_scroll_id(), scrollable::AbsoluteOffset { x, y: 0.0 })
    }

    pub fn close_workspace(&mut self, idx: usize) -> Task<Message> {
        if idx >= self.workspaces.len() { return Task::none(); }
        self.ws_menu_idx = None;
        let active_id = self.workspaces.get(self.active_ws).map(|w| w.id);
        let ws = self.workspaces.remove(idx);
        self.shutdown_lsp_for_workspace(ws.id);
        for pane_id in &ws.strip.panes { self.panes.remove(pane_id); }
        self.ws_colors.remove(&ws.id);
        if self.workspaces.is_empty() { self.active_ws = 0; return Task::none(); }
        self.active_ws = active_id
            .and_then(|id| self.workspaces.iter().position(|w| w.id == id))
            .unwrap_or(self.active_ws.min(self.workspaces.len() - 1));
        Task::none()
    }

    pub fn remove_pane(&mut self, pid: PaneId) -> Task<Message> {
        self.detach_lsp_for_pane(pid);
        self.pending_backends.retain(|pending| pending.pane_id != pid);
        self.panes.remove(&pid);
        let active_id = self.workspaces.get(self.active_ws).map(|w| w.id);
        for ws in &mut self.workspaces {
            ws.strip.remove(pid);
            if ws.preview_pane == Some(pid) {
                ws.preview_pane = None;
            }
        }
        let empty: Vec<WorkspaceId> = self.workspaces.iter()
            .filter(|ws| ws.is_empty()).map(|ws| ws.id).collect();
        for id in &empty { self.ws_colors.remove(id); }
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
