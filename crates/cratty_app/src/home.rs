use iced::widget::{button, column, container, row, text, Space};
use iced::{alignment, Color, Element, Length};

use crate::message::Message;
use crate::style::*;
use crate::widgets::*;

const ACCENT: Color = Color::from_rgb(0.30, 0.65, 0.90);

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
    .on_press(Message::NewTab)
    .padding([8, 20])
    .style(action_btn_style);

    let shortcut_hint = text("Ctrl+Shift+T").size(11).color(FG_DIM);

    let content = column![
        logo,
        subtitle,
        Space::new().height(32),
        new_tab_btn,
        Space::new().height(8),
        shortcut_hint,
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
        button::Status::Hovered => Color::from_rgb(0.18, 0.18, 0.18),
        _ => Color::from_rgb(0.13, 0.13, 0.13),
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: FG_ACTIVE,
        border: iced::Border {
            color: FG_MUTED,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}
