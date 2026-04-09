use iced::alignment;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Color, Element, Length};

use cratty_core::WorkspaceId;

use crate::message::Message;
use crate::style::*;
use crate::widgets::*;
use crate::ws_menu;

pub fn view_ws_item(
    idx: usize, ws_id: WorkspaceId, title: &str,
    color: Option<Color>, pane_count: usize, active: bool,
    show_menu: bool,
) -> Element<'_, Message> {
    let bar_w = if active { 4 } else { 3 };
    let accent_bar: Element<Message> = container(Space::new())
        .width(bar_w)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(
                color.unwrap_or(Color::TRANSPARENT),
            )),
            ..Default::default()
        })
        .into();

    let count_suffix = if pane_count > 1 {
        format!(" ({})", pane_count)
    } else {
        String::new()
    };
    let max_title = 15_usize.saturating_sub(count_suffix.len());
    let label = truncate(title, max_title);

    let menu_msg = if show_menu { Message::HideWsMenu } else { Message::ShowWsMenu(idx) };
    let menu_btn = button(centered_icon(ICO_DOTS_THREE, 10.0))
        .on_press(menu_msg)
        .width(20)
        .height(20)
        .padding(0)
        .style(|_, status| button::Style {
            text_color: match status {
                button::Status::Hovered => FG_ACTIVE,
                _ => FG_DIM,
            },
            ..Default::default()
        });

    let content = row![
        accent_bar,
        Space::new().width(8),
        text(format!("{}{}", label, count_suffix))
            .size(12)
            .color(if active { FG_ACTIVE } else { FG_INACTIVE }),
        Space::new().width(Length::Fill),
        menu_btn,
    ]
    .align_y(alignment::Vertical::Center)
    .height(32);

    let bg = if active { BG_WS_ACTIVE } else { BG_TITLEBAR };

    let item = button(content)
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
        });

    if show_menu {
        column![item, ws_menu::view_ws_menu(idx, ws_id)]
            .spacing(2).width(Length::Fill).into()
    } else {
        item.into()
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let take = max.saturating_sub(3);
        format!("{}...", s.chars().take(take).collect::<String>())
    } else {
        s.to_string()
    }
}
