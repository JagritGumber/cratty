use iced::window;
use iced::Task;

use crate::message::Message;
use crate::{with_window, Cratty};

impl Cratty {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewWorkspace => self.create_workspace(None),
            Message::CloseWorkspace(idx) => self.close_workspace(idx),
            Message::SwitchWorkspace(idx) => {
                if idx >= self.workspaces.len() { return Task::none(); }
                self.active_ws = idx;
                Task::none()
            }
            Message::NewPane => self.add_pane_to_active_workspace(None),
            Message::ClosePane(pid) => self.remove_pane(pid),
            Message::FocusPaneLeft => {
                if let Some(ws) = self.workspaces.get_mut(self.active_ws) {
                    ws.strip.focus_left();
                    let idx = ws.strip.focus_idx as f32;
                    ws.strip.viewport.animate_to(idx);
                }
                Task::none()
            }
            Message::FocusPaneRight => {
                if let Some(ws) = self.workspaces.get_mut(self.active_ws) {
                    ws.strip.focus_right();
                    let idx = ws.strip.focus_idx as f32;
                    ws.strip.viewport.animate_to(idx);
                }
                Task::none()
            }
            Message::ToggleSidebar => {
                self.sidebar_collapsed = !self.sidebar_collapsed;
                Task::none()
            }
            Message::RenameWorkspace(ws_id) => {
                if let Some(ws) = self.workspaces.iter().find(|w| w.id == ws_id) {
                    self.rename_text = ws.name.clone();
                }
                self.renaming_ws = Some(ws_id);
                self.ws_menu_idx = None;
                Task::none()
            }
            Message::RenameInput(text) => {
                self.rename_text = text;
                Task::none()
            }
            Message::RenameSubmit => {
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
            Message::SetWorkspaceColor(ws_id, color) => {
                self.ws_colors.insert(ws_id, color);
                self.ws_menu_idx = None;
                Task::none()
            }
            Message::ShowWsMenu(idx) => {
                self.ws_menu_idx = Some(idx);
                Task::none()
            }
            Message::HideWsMenu => {
                self.ws_menu_idx = None;
                Task::none()
            }
            Message::EscapePressed => {
                self.ws_menu_idx = None;
                self.renaming_ws = None;
                self.rename_text.clear();
                Task::none()
            }
            Message::Tick => {
                self.poll_pending_backends();
                self.tick_animations();
                self.apply_pending_resizes();
                self.process_terminal_events();
                Task::none()
            }
            Message::DragWindow => with_window(window::drag),
            Message::Minimize => with_window(|id| window::minimize(id, true)),
            Message::Maximize => with_window(window::toggle_maximize),
            Message::CloseWindow => with_window(window::close),
        }
    }
}
