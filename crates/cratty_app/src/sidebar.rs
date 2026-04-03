use std::collections::HashMap;

use iced::alignment;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Color, Element, Length};

use cratty_core::{PaneId, Workspace, WorkspaceId};

use crate::message::Message;
use crate::pane::Pane;
use crate::style::*;
use crate::widgets::*;

pub const SIDEBAR_W: f32 = 200.0;

pub fn view_sidebar<'a>(
    workspaces: &'a [Workspace], colors: &'a HashMap<WorkspaceId, Color>,
    panes: &'a HashMap<PaneId, Pane>, active: usize,
) -> Element<'a, Message> {
    let mut items: Vec<Element<Message>> = workspaces
        .iter()
        .enumerate()
        .map(|(idx, ws)| {
            let title = ws.focused_pane()
                .and_then(|pid| panes.get(&pid))
                .map(|p| p.display_title())
                .unwrap_or(&ws.name);
            let color = colors.get(&ws.id).copied();
            let pane_count = ws.strip.panes.len();
            view_ws_item(idx, ws.id, title, color, pane_count, idx == active)
        })
        .collect();

    items.push(new_ws_button());

    let list = scrollable(
        column(items).spacing(2).padding(6).width(Length::Fill),
    )
    .height(Length::Fill);

    container(list)
        .width(SIDEBAR_W)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(BG_TITLEBAR)),
            border: iced::Border {
                color: FG_MUTED,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn view_ws_item(
    idx: usize, _ws_id: WorkspaceId, title: &str,
    color: Option<Color>, pane_count: usize, active: bool,
) -> Element<'_, Message> {
    let accent_bar: Element<Message> = container(Space::new())
        .width(3)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(
                color.unwrap_or(Color::TRANSPARENT),
            )),
            ..Default::default()
        })
        .into();

    let label = truncate(title, 18);
    let count_label = if pane_count > 1 {
        format!(" ({})", pane_count)
    } else {
        String::new()
    };

    let content = row![
        accent_bar,
        Space::new().width(8),
        text(format!("{}{}", label, count_label))
            .size(12)
            .color(if active { FG_ACTIVE } else { FG_INACTIVE }),
    ]
    .align_y(alignment::Vertical::Center)
    .height(32);

    let bg = if active { BG_MENU_HOVER } else { BG_TITLEBAR };
    button(content)
        .on_press(Message::SwitchWorkspace(idx))
        .padding([0, 4])
        .width(Length::Fill)
        .style(move |_, status| button::Style {
            background: Some(iced::Background::Color(match status {
                button::Status::Hovered => BG_MENU_HOVER,
                _ => bg,
            })),
            border: iced::Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

fn new_ws_button() -> Element<'static, Message> {
    button(
        row![
            centered_icon(ICO_PLUS, 10.0),
            text("New workspace").size(11).color(FG_DIM),
        ]
        .spacing(6)
        .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::NewWorkspace)
    .padding([6, 8])
    .width(Length::Fill)
    .style(|_, status| button::Style {
        background: match status {
            button::Status::Hovered => Some(iced::Background::Color(BG_MENU_HOVER)),
            _ => None,
        },
        text_color: match status {
            button::Status::Hovered => FG_ACTIVE,
            _ => FG_DIM,
        },
        border: iced::Border { radius: 4.0.into(), ..Default::default() },
        ..Default::default()
    })
    .into()
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        format!("{}...", s.chars().take(max - 3).collect::<String>())
    } else {
        s.to_string()
    }
}
