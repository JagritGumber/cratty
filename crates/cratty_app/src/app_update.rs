use alacritty_terminal::grid::Scroll;
use iced::widget::{operation, scrollable};
use iced::{window, Task};

use crate::message::Message;
use crate::strip_view::strip_scroll_id;
use crate::{with_window, Cratty};

impl Cratty {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewWorkspace => self.create_workspace(None),
            Message::CloseWorkspace(idx) => self.close_workspace(idx),
            Message::SwitchWorkspace(idx) => {
                if idx >= self.workspaces.len() { return Task::none(); }
                if let Some(ws) = self.workspaces.get_mut(self.active_ws) {
                    ws.strip.saved_scroll_x = compute_scroll_x(&ws.strip, self.viewport_w);
                }
                self.active_ws = idx;
                let x = self.workspaces.get(self.active_ws)
                    .map_or(0.0, |ws| ws.strip.saved_scroll_x);
                operation::scroll_to(strip_scroll_id(), scrollable::AbsoluteOffset { x, y: 0.0 })
            }
            Message::NewPane => self.add_pane_to_active_workspace(None),
            Message::ClosePane(pid) => self.remove_pane(pid),
            Message::CloseFocusedPane => {
                let pid = self.workspaces.get(self.active_ws).and_then(|ws| ws.focused_pane());
                if let Some(pid) = pid { self.remove_pane(pid) } else { Task::none() }
            }
            Message::MovePaneLeft | Message::MovePaneRight => {
                let ws = match self.workspaces.get_mut(self.active_ws) { Some(w) => w, None => return Task::none() };
                let moved = if matches!(message, Message::MovePaneLeft) { ws.strip.swap_left() } else { ws.strip.swap_right() };
                if moved { scroll_to_pane(&ws.strip, self.viewport_w) } else { Task::none() }
            }
            Message::FocusPaneLeft | Message::FocusPaneRight => {
                let ws = match self.workspaces.get_mut(self.active_ws) { Some(w) => w, None => return Task::none() };
                let moved = if matches!(message, Message::FocusPaneLeft) { ws.strip.focus_left() } else { ws.strip.focus_right() };
                if moved { ws.strip.viewport.animate_to(ws.strip.focus_idx as f32); scroll_to_pane(&ws.strip, self.viewport_w) } else { Task::none() }
            }
            Message::CyclePresetWidth | Message::ToggleMaximizePane
            | Message::GrowPane | Message::ShrinkPane => {
                if let Some(ws) = self.workspaces.get_mut(self.active_ws) {
                    match message {
                        Message::CyclePresetWidth => ws.strip.cycle_preset_width(),
                        Message::ToggleMaximizePane => ws.strip.toggle_maximize(),
                        Message::GrowPane => ws.strip.adjust_focused_width(0.1),
                        _ => ws.strip.adjust_focused_width(-0.1),
                    }
                    return scroll_to_pane(&ws.strip, self.viewport_w);
                }
                Task::none()
            }
            Message::ScrollTermUp => { self.scroll_focused_term(Scroll::PageUp); Task::none() }
            Message::ScrollTermDown => { self.scroll_focused_term(Scroll::PageDown); Task::none() }
            Message::CopyTerminal => { self.copy_focused_term(); Task::none() }
            Message::PasteTerminal => { self.paste_to_focused_term(); Task::none() }
            Message::ToggleSidebar => {
                self.sidebar_collapsed = !self.sidebar_collapsed; Task::none()
            }
            Message::RenameWorkspace(ws_id) => self.handle_rename_ws(ws_id),
            Message::RenameInput(text) => { self.rename_text = text; Task::none() }
            Message::RenameSubmit => self.handle_rename_submit(),
            Message::SetWorkspaceColor(ws_id, color) => {
                self.ws_colors.insert(ws_id, color); self.ws_menu_idx = None; Task::none()
            }
            Message::ShowWsMenu(idx) => { self.ws_menu_idx = Some(idx); Task::none() }
            Message::HideWsMenu => { self.ws_menu_idx = None; Task::none() }
            Message::EscapePressed => self.handle_escape(),
            Message::Tick => {
                self.poll_pending_backends(); self.tick_animations();
                self.apply_pending_resizes(); self.process_terminal_events();
                Task::none()
            }
            Message::WindowResized(size) => { self.viewport_w = size.width; Task::none() }
            Message::DragWindow => with_window(window::drag),
            Message::Minimize => with_window(|id| window::minimize(id, true)),
            Message::Maximize => with_window(window::toggle_maximize),
            Message::CloseWindow => with_window(window::close),
        }
    }
}

fn compute_scroll_x(strip: &cratty_core::PaperStrip, vw: f32) -> f32 {
    let start: f32 = (0..strip.focus_idx)
        .map(|i| strip.pane_width_at(i, vw) + 2.0).sum();
    (start - (vw - strip.pane_width_at(strip.focus_idx, vw)) / 2.0).max(0.0)
}

fn scroll_to_pane(strip: &cratty_core::PaperStrip, vw: f32) -> Task<Message> {
    let x = compute_scroll_x(strip, vw);
    operation::scroll_to(strip_scroll_id(), scrollable::AbsoluteOffset { x, y: 0.0 })
}
