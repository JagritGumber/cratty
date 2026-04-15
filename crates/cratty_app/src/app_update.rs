use alacritty_terminal::grid::Scroll;
use iced::widget::{operation, scrollable};
use iced::{window, Task};
use crate::message::Message;
use crate::strip_view::strip_scroll_id;
use crate::{with_window, Cratty};

impl Cratty {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewWorkspace => Task::perform(
                async { rfd::AsyncFileDialog::new().set_title("Pick project folder").pick_folder().await },
                |opt| Message::WorkspaceFolderPicked(opt.map(|h| h.path().to_path_buf())),
            ),
            Message::WorkspaceFolderPicked(None) => Task::none(),
            Message::WorkspaceFolderPicked(Some(path)) => self.create_workspace_with_root(path),
            Message::CloseWorkspace(idx) => self.close_workspace(idx),
            Message::SwitchWorkspace(idx) => self.switch_workspace(idx),
            Message::NewPane => self.add_pane_to_active_workspace(None),
            Message::ClosePane(pid) => self.remove_pane(pid),
            Message::CloseFocusedPane => {
                let pid = self.workspaces.get(self.active_ws).and_then(|ws| ws.focused_pane());
                if let Some(pid) = pid { self.remove_pane(pid) } else { Task::none() }
            }
            Message::MovePaneLeft | Message::MovePaneRight => {
                let ws = match self.workspaces.get_mut(self.active_ws) { Some(w) => w, None => return Task::none() };
                let moved = if matches!(message, Message::MovePaneLeft) { ws.strip.swap_left() } else { ws.strip.swap_right() };
                if moved {
                    ws.strip.viewport.animate_to(compute_scroll_x(&ws.strip, self.viewport_w));
                    return self.focus_focused_editor_pane();
                }
                Task::none()
            }
            Message::FocusPaneLeft | Message::FocusPaneRight => {
                let ws = match self.workspaces.get_mut(self.active_ws) { Some(w) => w, None => return Task::none() };
                let moved = if matches!(message, Message::FocusPaneLeft) { ws.strip.focus_left() } else { ws.strip.focus_right() };
                if moved {
                    ws.strip.viewport.animate_to(compute_scroll_x(&ws.strip, self.viewport_w));
                    return self.focus_focused_editor_pane();
                }
                Task::none()
            }
            Message::CyclePresetWidth | Message::ToggleMaximizePane | Message::GrowPane | Message::ShrinkPane => {
                let vw = self.viewport_w;
                if let Some(ws) = self.workspaces.get_mut(self.active_ws) {
                    match message {
                        Message::CyclePresetWidth => ws.strip.cycle_preset_width(vw),
                        Message::ToggleMaximizePane => ws.strip.toggle_maximize(vw),
                        Message::GrowPane => ws.strip.adjust_focused_width(0.1, vw),
                        _ => ws.strip.adjust_focused_width(-0.1, vw),
                    }
                    ws.strip.viewport.animate_to(compute_scroll_x(&ws.strip, vw));
                }
                Task::none()
            }
            Message::ScrollTermUp => { self.scroll_focused_term(Scroll::PageUp); Task::none() }
            Message::ScrollTermDown => { self.scroll_focused_term(Scroll::PageDown); Task::none() }
            Message::CopyTerminal => { self.copy_focused_term(); Task::none() }
            Message::PasteTerminal => { self.paste_to_focused_term(); Task::none() }
            Message::ToggleSidebar => { self.sidebar_collapsed = !self.sidebar_collapsed; Task::none() }
            Message::RenameWorkspace(ws_id) => self.handle_rename_ws(ws_id),
            Message::RenameInput(text) => { self.rename_text = text; Task::none() }
            Message::RenameSubmit => self.handle_rename_submit(),
            Message::SetWorkspaceColor(ws_id, color) => {
                self.ws_colors.insert(ws_id, color); self.ws_menu_idx = None; self.color_submenu = false; Task::none()
            }
            Message::ShowWsMenu(idx) => { self.ws_menu_idx = Some(idx); self.color_submenu = false; Task::none() }
            Message::HideWsMenu => { self.ws_menu_idx = None; self.color_submenu = false; Task::none() }
            Message::ToggleColorSubmenu => { self.color_submenu = !self.color_submenu; Task::none() }
            Message::EscapePressed => self.handle_escape(),
            Message::Tick => {
                self.poll_pending_backends();
                let scroll_x = self.tick_animations();
                self.apply_pending_resizes(); self.process_terminal_events();
                match scroll_x {
                    Some(x) => operation::scroll_to(strip_scroll_id(), scrollable::AbsoluteOffset { x, y: 0.0 }),
                    None => Task::none(),
                }
            }
            Message::WindowResized(size) => { self.viewport_w = size.width; Task::none() }
            Message::DragWindow => with_window(window::drag),
            Message::Minimize => with_window(|id| window::minimize(id, true)),
            Message::Maximize => with_window(window::toggle_maximize),
            Message::CloseWindow => { self.save_layout(); with_window(window::close) }
            Message::ToggleQuake => self.handle_toggle_quake(),
            Message::CodeAction(pid, action) => self.handle_code_action(pid, action),
            Message::SaveFocusedFile => self.save_focused_file(),
            Message::OpenFilePicker => self.open_file_picker(),
            Message::FilePickerInput(q) => self.file_picker_input(q),
            Message::FilePickerMove(d) => self.file_picker_move(d),
            Message::FilePickerSelect(idx) => self.file_picker_select(idx),
            Message::FilePickerOpen => self.file_picker_open(),
            Message::FilePickerClose => self.file_picker_close(),
        }
    }
}

pub fn compute_scroll_x(strip: &cratty_core::PaperStrip, vw: f32) -> f32 {
    let start: f32 = (0..strip.focus_idx)
        .map(|i| strip.pane_width_at(i, vw) + 2.0).sum();
    (start - (vw - strip.pane_width_at(strip.focus_idx, vw)) / 2.0).max(0.0)
}

