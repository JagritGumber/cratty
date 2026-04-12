use iced::widget::{button, column, container, row, text, Space};
use iced::{Color, Element, Length};
use cratty_core::WorkspaceId;
use crate::message::Message;
use crate::style::*;
use crate::widgets::{ICO_CARET_RIGHT, PHOSPHOR};

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
    let color_item = color_menu_trigger();
    let delete_btn = menu_item("Delete", Message::CloseWorkspace(idx));

    let content = column![rename_btn, color_item, delete_btn]
        .spacing(2).padding(4).width(160);

    container(content).style(menu_container_style).into()
}

fn color_menu_trigger() -> Element<'static, Message> {
    let label = row![
        text("Color").size(12),
        Space::new().width(Length::Fill),
        text(ICO_CARET_RIGHT).font(PHOSPHOR).size(10).color(FG_DIM)
            .shaping(text::Shaping::Advanced),
    ]
    .align_y(iced::alignment::Vertical::Center);

    button(label).on_press(Message::ToggleColorSubmenu)
        .padding([6, 12]).width(Length::Fill)
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

pub fn view_color_submenu(ws_id: WorkspaceId) -> Element<'static, Message> {
    let swatches = row(PRESET_COLORS.iter().map(|(c, _)| color_swatch(*c, ws_id))).spacing(6);
    container(swatches).padding(8).style(menu_container_style).into()
}

fn menu_item(label: &str, msg: Message) -> Element<'static, Message> {
    button(text(label.to_string()).size(12))
        .on_press(msg).padding([6, 12]).width(Length::Fill)
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
        .width(18).height(18).padding(0)
        .style(move |_, status| button::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border {
                color: match status { button::Status::Hovered => FG_ACTIVE, _ => Color::TRANSPARENT },
                width: 2.0, radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn menu_container_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(BG_TITLEBAR)),
        border: iced::Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            width: 1.0, radius: 8.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
            offset: iced::Vector::new(0.0, 10.0),
            blur_radius: 25.0,
        },
        ..Default::default()
    }
}
