use iced::widget::{button, column, container, row, text, Space};
use iced::{alignment, Element, Length};

use crate::message::Message;
use crate::style::*;
use crate::widgets::*;

use crate::style::ACCENT;

pub fn view_home<'a>() -> Element<'a, Message> {
    let logo = text("Cratty")
        .size(42)
        .color(FG_ACTIVE)
        .align_x(alignment::Horizontal::Center);

    let subtitle = text("Terminal Emulator")
        .size(14)
        .color(FG_INACTIVE)
        .align_x(alignment::Horizontal::Center);

    let new_tab_btn = button(
        row![
            text(ICO_PLUS).font(PHOSPHOR).size(14).color(ACCENT),
            text("New Terminal").size(13).color(FG_ACTIVE),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::NewWorkspace)
    .padding([8, 20])
    .style(action_btn_style);

    let shortcut_hint = text("Alt+T").size(11).color(FG_DIM);

    let shortcuts_guide = shortcuts_table();

    let content = column![
        logo,
        subtitle,
        Space::new().height(32),
        new_tab_btn,
        Space::new().height(8),
        shortcut_hint,
        Space::new().height(24),
        shortcuts_guide,
    ]
    .align_x(alignment::Horizontal::Center)
    .spacing(4);

    container(content)
        .center(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(BG_TERMINAL)),
            ..Default::default()
        })
        .into()
}

fn action_btn_style(_: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => BG_WS_ACTIVE,
        _ => BG_MENU_HOVER,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: FG_ACTIVE,
        border: iced::Border { radius: 6.0.into(), ..Default::default() },
        ..Default::default()
    }
}

fn shortcuts_table<'a>() -> Element<'a, Message> {
    let pairs = [
        ("Alt+T", "New workspace"),
        ("Alt+N", "New pane"),
        ("Alt+B", "Toggle sidebar"),
        ("Alt+R", "Cycle pane width"),
        ("Alt+F", "Maximize pane"),
    ];
    let rows: Vec<Element<Message>> = pairs
        .iter()
        .map(|(key, desc)| shortcut_row(key, desc))
        .collect();
    column(rows).spacing(4).into()
}

fn shortcut_row<'a>(key: &str, desc: &str) -> Element<'a, Message> {
    row![
        text(key.to_string()).size(11).color(FG_INACTIVE).width(90),
        text(desc.to_string()).size(11).color(FG_DIM),
    ]
    .spacing(8)
    .into()
}
