use std::collections::HashMap;

use iced::alignment;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Color, Element, Length};

use cratty_core::{PaneId, Workspace, WorkspaceId};

use crate::message::Message;
use crate::pane::Pane;
use crate::style::*;
use crate::widgets::*;
use crate::ws_item;

pub const SIDEBAR_W: f32 = 200.0;

pub fn view_sidebar<'a>(
    workspaces: &'a [Workspace], colors: &'a HashMap<WorkspaceId, Color>,
    panes: &'a HashMap<PaneId, Pane>, active: usize,
    renaming: Option<WorkspaceId>, rename_text: &'a str,
    menu_idx: Option<usize>,
) -> Element<'a, Message> {
    let header = row![
        text("Workspaces").size(11).color(FG_DIM),
        Space::new().width(Length::Fill),
        collapse_btn(true),
    ]
    .align_y(alignment::Vertical::Center)
    .padding([6, 8]);

    let mut items: Vec<Element<Message>> = workspaces.iter().enumerate()
        .map(|(idx, ws)| {
            if renaming == Some(ws.id) {
                return rename_input(rename_text);
            }
            let title = ws.focused_pane()
                .and_then(|pid| panes.get(&pid))
                .map(|p| p.display_title())
                .unwrap_or(&ws.name);
            let color = colors.get(&ws.id).copied();
            let show_menu = menu_idx == Some(idx);
            ws_item::view_ws_item(
                idx, ws.id, title, color, ws.strip.panes.len(),
                idx == active, show_menu,
            )
        })
        .collect();
    items.push(new_ws_button());

    let list = scrollable(column(items).spacing(2).padding(6).width(Length::Fill))
        .height(Length::Fill);

    container(column![header, list])
        .width(SIDEBAR_W).height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(BG_TITLEBAR)),
            ..Default::default()
        })
        .into()
}

pub fn collapse_btn(is_expanded: bool) -> Element<'static, Message> {
    let icon = if is_expanded { ICO_CARET_LEFT } else { ICO_CARET_RIGHT };
    button(centered_icon(icon, 10.0))
        .on_press(Message::ToggleSidebar)
        .width(24).height(24).padding(0)
        .style(|_, status| button::Style {
            text_color: match status {
                button::Status::Hovered => FG_ACTIVE, _ => FG_DIM,
            },
            ..Default::default()
        })
        .into()
}

fn rename_input(value: &str) -> Element<'_, Message> {
    text_input("Name...", value)
        .on_input(Message::RenameInput)
        .on_submit(Message::RenameSubmit)
        .size(12).padding([4, 8]).width(Length::Fill)
        .into()
}

fn new_ws_button() -> Element<'static, Message> {
    button(
        row![centered_icon(ICO_PLUS, 10.0), text("New workspace").size(11).color(FG_DIM)]
            .spacing(6).align_y(alignment::Vertical::Center),
    )
    .on_press(Message::NewWorkspace)
    .padding([6, 8]).width(Length::Fill)
    .style(|_, status| {
        let hovered = matches!(status, button::Status::Hovered);
        button::Style {
            background: if hovered { Some(iced::Background::Color(BG_MENU_HOVER)) } else { None },
            text_color: if hovered { FG_ACTIVE } else { FG_DIM },
            border: iced::Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        }
    })
    .into()
}
