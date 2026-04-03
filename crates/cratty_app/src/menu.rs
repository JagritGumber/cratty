use iced::widget::{button, column, container, mouse_area, row, text, Space};
use iced::{Color, Element, Length};

use cratty_core::WorkspaceId;

use crate::message::Message;
use crate::style::*;
use crate::widgets::*;

pub fn view_tab_menu(idx: usize, color_submenu_open: bool) -> Element<'static, Message> {
    let color_label = row![
        text("Color").size(12).color(FG_ACTIVE),
        Space::new().width(Length::Fill),
        text("\u{203A}").size(14).color(FG_DIM),
    ]
    .width(Length::Fill)
    .padding([6, 16]);

    let color_bg = if color_submenu_open { BG_MENU_HOVER } else { BG_MENU };
    let color_item: Element<Message> = mouse_area(
        container(color_label)
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(color_bg)),
                ..Default::default()
            }),
    )
    .on_enter(Message::OpenColorSubmenu)
    .into();

    container(
        column![
            menu_item("Rename", Message::StartRename(idx)),
            menu_item("Duplicate", Message::DuplicateWorkspace(idx)),
            color_item,
            menu_item("Close", Message::CloseWorkspace(idx)),
        ]
        .width(140),
    )
    .style(|_| container::Style {
        background: Some(iced::Background::Color(BG_MENU)),
        border: iced::Border { color: FG_MUTED, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    })
    .into()
}

pub fn view_color_panel(ws_id: WorkspaceId, current: Option<Color>) -> Element<'static, Message> {
    let make_swatch = |color: Color, selected: bool| -> Element<'static, Message> {
        button(Space::new().width(18).height(18))
            .on_press(Message::SetWorkspaceColor(ws_id, Some(color)))
            .width(24).height(24).padding(3)
            .style(move |_, status| {
                let border = if selected {
                    iced::Border { color: FG_ACTIVE, width: 2.0, radius: 4.0.into() }
                } else {
                    match status {
                        button::Status::Hovered => iced::Border {
                            color: FG_DIM, width: 1.0, radius: 4.0.into(),
                        },
                        _ => iced::Border { radius: 4.0.into(), ..Default::default() },
                    }
                };
                button::Style {
                    background: Some(iced::Background::Color(color)),
                    border,
                    ..Default::default()
                }
            })
            .into()
    };

    let r1: Vec<Element<Message>> = TAB_COLORS[..4].iter()
        .map(|(c, _)| make_swatch(*c, current == Some(*c))).collect();
    let mut r2: Vec<Element<Message>> = TAB_COLORS[4..].iter()
        .map(|(c, _)| make_swatch(*c, current == Some(*c))).collect();

    if current.is_some() {
        r2.push(
            button(phosphor_icon(ICO_X, 14.0, Color::from_rgb(0.85, 0.2, 0.2)))
                .on_press(Message::SetWorkspaceColor(ws_id, None))
                .width(24).height(24)
                .padding(iced::Padding { top: 1.0, right: 0.0, bottom: 0.0, left: 1.0 })
                .style(|_, status| button::Style {
                    background: Some(iced::Background::Color(match status {
                        button::Status::Hovered => Color::from_rgb(0.2, 0.1, 0.1),
                        _ => Color::from_rgb(0.12, 0.12, 0.12),
                    })),
                    border: iced::Border {
                        color: Color::from_rgb(0.3, 0.15, 0.15), width: 1.0, radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .into(),
        );
    }

    container(column![row(r1).spacing(3), row(r2).spacing(3)].spacing(3).padding(6))
        .style(|_| container::Style {
            background: Some(iced::Background::Color(BG_MENU)),
            border: iced::Border { color: FG_MUTED, width: 1.0, radius: 4.0.into() },
            ..Default::default()
        })
        .into()
}
