use iced::widget::{container, mouse_area, opaque, stack, Space};
use iced::{Element, Length, Task};
use crate::message::Message;
use crate::sidebar::SIDEBAR_W;
use crate::style::TITLEBAR_H;
use crate::{ws_menu, Cratty};

const MENU_W: f32 = 160.0;
const HEADER_H: f32 = 36.0;
const ITEM_STRIDE: f32 = 38.0;

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
                    ws.auto_named = false;
                }
            }
        }
        self.rename_text.clear();
        Task::none()
    }

    pub fn handle_escape(&mut self) -> Task<Message> {
        if self.file_picker.is_some() { self.file_picker = None; return Task::none(); }
        self.ws_menu_idx = None; self.renaming_ws = None;
        self.color_submenu = false; self.rename_text.clear(); Task::none()
    }

    pub fn switch_workspace(&mut self, idx: usize) -> Task<Message> {
        self.ws_menu_idx = None;
        if idx >= self.workspaces.len() { return Task::none(); }
        if let Some(ws) = self.workspaces.get_mut(self.active_ws) {
            ws.strip.saved_scroll_x = crate::app_update::compute_scroll_x(&ws.strip, self.viewport_w);
        }
        self.active_ws = idx;
        let x = self.workspaces.get(self.active_ws).map_or(0.0, |ws| ws.strip.saved_scroll_x);
        if let Some(ws) = self.workspaces.get_mut(self.active_ws) {
            ws.strip.viewport = cratty_core::ViewOffset::Static(x);
        }
        iced::widget::operation::scroll_to(
            crate::strip_view::strip_scroll_id(),
            iced::widget::scrollable::AbsoluteOffset { x, y: 0.0 },
        )
    }

    pub fn menu_overlay(&self) -> Option<Element<'_, Message>> {
        if self.sidebar_collapsed { return None; }
        let idx = self.ws_menu_idx?;
        let ws = self.workspaces.get(idx)?;

        let y = TITLEBAR_H + HEADER_H + (idx as f32) * ITEM_STRIDE + ITEM_STRIDE;
        let x = SIDEBAR_W - 28.0;

        // Backdrop: dismisses on click, blocks hover via opaque menu on top
        let backdrop = mouse_area(
            container(Space::new()).width(Length::Fill).height(Length::Fill),
        ).on_press(Message::HideWsMenu);

        // Main menu: only the menu content is opaque, not the positioning container
        let menu = positioned(opaque(ws_menu::view_ws_menu(idx, ws.id)), x, y);

        let mut layers: Vec<Element<Message>> = vec![backdrop.into(), menu];

        // Color submenu: same pattern, opaque on content only
        if self.color_submenu {
            let sub = positioned(
                opaque(ws_menu::view_color_submenu(ws.id)),
                x + MENU_W + 4.0,
                y + 36.0,
            );
            layers.push(sub);
        }

        Some(stack(layers).width(Length::Fill).height(Length::Fill).into())
    }
}

fn positioned<'a>(content: Element<'a, Message>, x: f32, y: f32) -> Element<'a, Message> {
    container(content)
        .padding(iced::Padding { top: y, left: x, ..Default::default() })
        .width(Length::Fill).height(Length::Fill).into()
}
