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
        text("Workspaces").size(13).color(FG_INACTIVE),
        Space::new().width(Length::Fill),
    ]
    .align_y(alignment::Vertical::Center)
    .padding([8, 10]);

    let mut items: Vec<Element<Message>> = workspaces.iter().enumerate()
        .map(|(idx, ws)| {
            if renaming == Some(ws.id) {
                return rename_input(rename_text);
            }
            let title = display_name(ws, panes);
            let color = colors.get(&ws.id).copied();
            let show_menu = menu_idx == Some(idx);
            ws_item::view_ws_item(
                idx, title, color, ws.strip.panes.len(),
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

/// Priority: user's custom name > CWD basename > workspace default name.
fn display_name(ws: &Workspace, panes: &HashMap<PaneId, Pane>) -> String {
    if !ws.auto_named { return ws.name.clone(); }
    ws.focused_pane().and_then(|pid| panes.get(&pid))
        .and_then(|p| p.cwd.as_ref())
        .and_then(|c| c.file_name().and_then(|n| n.to_str()).map(String::from))
        .unwrap_or_else(|| ws.name.clone())
}

fn rename_input(value: &str) -> Element<'_, Message> {
    text_input("Name...", value)
        .on_input(Message::RenameInput)
        .on_submit(Message::RenameSubmit)
        .size(12).padding([4, 8]).width(Length::Fill)
        .into()
}

fn new_ws_button() -> Element<'static, Message> {
    let icon = container(
        text(ICO_PLUS).font(crate::widgets::PHOSPHOR).size(10)
            .shaping(iced::widget::text::Shaping::Advanced),
    )
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center)
    .width(Length::Shrink);

    button(
        row![icon, text("New workspace").size(13).color(FG_DIM)]
            .spacing(8).align_y(alignment::Vertical::Center),
    )
    .on_press(Message::NewWorkspace)
    .padding([8, 10]).width(Length::Fill)
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
