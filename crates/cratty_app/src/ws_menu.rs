use iced::widget::{button, column, container, row, text, Space};
use iced::{Color, Element, Length};

use cratty_core::WorkspaceId;

use crate::message::Message;
use crate::style::*;

const PRESET_COLORS: [(Color, &str); 6] = [
    (Color::from_rgb(0.90, 0.30, 0.30), "Red"),
    (Color::from_rgb(0.30, 0.80, 0.40), "Green"),
    (Color::from_rgb(0.30, 0.65, 0.90), "Blue"),
    (Color::from_rgb(0.90, 0.75, 0.20), "Yellow"),
    (Color::from_rgb(0.75, 0.40, 0.90), "Purple"),
    (Color::from_rgb(0.90, 0.55, 0.20), "Orange"),
];

pub fn view_ws_menu(idx: usize, ws_id: WorkspaceId) -> Element<'static, Message> {
    let rename_btn = menu_item("Rename", Message::RenameWorkspace(ws_id));
    let delete_btn = menu_item("Delete", Message::CloseWorkspace(idx));

    let color_row: Element<Message> = row(
        PRESET_COLORS.iter().map(|(color, _)| color_swatch(*color, ws_id)),
    )
    .spacing(4)
    .into();

    let content = column![
        rename_btn,
        text("Color").size(10).color(FG_DIM),
        color_row,
        Space::new().height(4),
        delete_btn,
    ]
    .spacing(4)
    .padding(8)
    .width(160);

    container(content)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(BG_TITLEBAR)),
            border: iced::Border {
                color: FG_MUTED,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn menu_item(label: &str, msg: Message) -> Element<'static, Message> {
    button(text(label.to_string()).size(12))
        .on_press(msg)
        .padding([4, 8])
        .width(Length::Fill)
        .style(|_, status| button::Style {
            background: match status {
                button::Status::Hovered => Some(iced::Background::Color(BG_MENU_HOVER)),
                _ => None,
            },
            text_color: FG_ACTIVE,
            border: iced::Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

fn color_swatch(color: Color, ws_id: WorkspaceId) -> Element<'static, Message> {
    button(Space::new())
        .on_press(Message::SetWorkspaceColor(ws_id, color))
        .width(18)
        .height(18)
        .padding(0)
        .style(move |_, status| button::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border {
                color: match status {
                    button::Status::Hovered => FG_ACTIVE,
                    _ => Color::TRANSPARENT,
                },
                width: 2.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}
