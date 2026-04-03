use iced::alignment;
use iced::widget::{button, mouse_area, row, text, text_input, Space};
use iced::{Color, Element, Length};

use crate::message::{Message, RenameState, Tab};
use crate::style::*;
use crate::widgets::*;

pub fn view_titlebar<'a>(
    tabs: &'a [Tab], active_tab: usize, renaming: &'a Option<RenameState>,
) -> Element<'a, Message> {
    let mut items: Vec<Element<Message>> = tabs
        .iter()
        .enumerate()
        .map(|(idx, tab)| view_tab(idx, tab, active_tab, renaming))
        .collect();

    items.push(icon_btn(ICO_PLUS, 12.0, FG_DIM, Message::NewTab));

    let tabs_row = row(items)
        .spacing(TAB_ROW_SPACING)
        .padding(iced::Padding { top: 0.0, right: 8.0, bottom: 0.0, left: TAB_ROW_LEFT })
        .align_y(alignment::Vertical::Center);

    let controls = row![
        win_btn(ICO_MINUS, Message::Minimize, FG_DIM, Color::from_rgb(0.2, 0.2, 0.2)),
        win_btn(ICO_SQUARE, Message::Maximize, FG_DIM, Color::from_rgb(0.2, 0.2, 0.2)),
        win_btn(ICO_X, Message::CloseWindow, FG_DIM, Color::from_rgb(0.7, 0.15, 0.15)),
    ]
    .spacing(0);

    let bar = row![tabs_row, Space::new().width(Length::Fill), controls]
        .align_y(alignment::Vertical::Center)
        .height(TITLEBAR_H);

    mouse_area(
        iced::widget::container(bar)
            .width(Length::Fill)
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(BG_TITLEBAR)),
                ..Default::default()
            }),
    )
    .on_press(Message::DragWindow)
    .into()
}

fn view_tab<'a>(
    idx: usize, tab: &'a Tab, active_tab: usize, renaming: &'a Option<RenameState>,
) -> Element<'a, Message> {
    let active = idx == active_tab;
    let is_renaming = renaming.as_ref().is_some_and(|r| r.tab_id == tab.id);
    let icon_fg = if active { FG_DIM } else { FG_MUTED };

    let tab_content: Element<Message> = if is_renaming {
        let input_val = renaming.as_ref().unwrap().input.clone();
        text_input("Tab name (empty to reset)", &input_val)
            .on_input(Message::RenameInput)
            .on_submit(Message::ConfirmRename)
            .size(11)
            .width(120)
            .padding([2, 4])
            .style(rename_input_style)
            .into()
    } else {
        let display = tab.display_title();
        let label: String = if display.chars().count() > 20 {
            format!("{}...", display.chars().take(17).collect::<String>())
        } else {
            display.to_string()
        };
        text(label)
            .size(11)
            .color(if active { FG_ACTIVE } else { FG_INACTIVE })
            .into()
    };

    let tab_row = row![
        tab_content,
        icon_btn(ICO_DOTS_THREE_V, 14.0, icon_fg, Message::ToggleTabMenu(tab.id)),
        icon_btn(ICO_X, 10.0, icon_fg, Message::CloseTab(idx)),
    ]
    .spacing(TAB_ICON_GAP)
    .align_y(alignment::Vertical::Center);

    let color = tab.color;
    button(tab_row)
        .on_press(Message::SwitchTab(idx))
        .padding([5, TAB_PAD_H as u16])
        .style(move |_, _| tab_style(active, color))
        .into()
}
