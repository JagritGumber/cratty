use iced::Task;

use crate::message::Message;
use crate::Cratty;

impl Cratty {
    pub fn handle_rename_ws(&mut self, ws_id: cratty_core::WorkspaceId) -> Task<Message> {
        if let Some(ws) = self.workspaces.iter().find(|w| w.id == ws_id) {
            self.rename_text = ws.name.clone();
        }
        self.renaming_ws = Some(ws_id); self.ws_menu_idx = None; Task::none()
    }

    pub fn handle_rename_submit(&mut self) -> Task<Message> {
        if let Some(ws_id) = self.renaming_ws.take() {
            if let Some(ws) = self.workspaces.iter_mut().find(|w| w.id == ws_id) {
                if !self.rename_text.is_empty() {
                    ws.name = self.rename_text.clone();
                }
            }
        }
        self.rename_text.clear();
        Task::none()
    }

    pub fn handle_escape(&mut self) -> Task<Message> {
        self.ws_menu_idx = None; self.renaming_ws = None;
        self.rename_text.clear(); Task::none()
    }
}
