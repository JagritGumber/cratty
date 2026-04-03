use std::collections::HashMap;

use iced::alignment;
use iced::widget::{button, mouse_area, row, text, text_input, Space};
use iced::{Color, Element, Length};

use cratty_core::{Workspace, WorkspaceId};

use crate::message::{Message, RenameState};
use crate::style::*;
use crate::widgets::*;

pub fn view_titlebar<'a>(
    workspaces: &'a [Workspace], colors: &'a HashMap<WorkspaceId, Color>,
    active: usize, renaming: &'a Option<RenameState>,
) -> Element<'a, Message> {
    let mut items: Vec<Element<Message>> = workspaces
        .iter()
        .enumerate()
        .map(|(idx, ws)| view_ws_tab(idx, ws, colors.get(&ws.id).copied(), active, renaming))
        .collect();

    items.push(icon_btn(ICO_PLUS, 12.0, FG_DIM, Message::NewWorkspace));

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

fn view_ws_tab<'a>(
    idx: usize, ws: &'a Workspace, color: Option<Color>,
    active_idx: usize, renaming: &'a Option<RenameState>,
) -> Element<'a, Message> {
    let active = idx == active_idx;
    let is_renaming = renaming.as_ref().is_some_and(|r| r.workspace_id == ws.id);
    let icon_fg = if active { FG_DIM } else { FG_MUTED };

    let tab_content: Element<Message> = if is_renaming {
        let input_val = renaming.as_ref().unwrap().input.clone();
        text_input("Name (empty to reset)", &input_val)
            .on_input(Message::RenameInput)
            .on_submit(Message::ConfirmRename)
            .size(11)
            .width(120)
            .padding([2, 4])
            .style(rename_input_style)
            .into()
    } else {
        let label = truncate_name(&ws.name, 20);
        text(label)
            .size(11)
            .color(if active { FG_ACTIVE } else { FG_INACTIVE })
            .into()
    };

    let tab_row = row![
        tab_content,
        icon_btn(ICO_DOTS_THREE_V, 14.0, icon_fg, Message::ToggleTabMenu(ws.id)),
        icon_btn(ICO_X, 10.0, icon_fg, Message::CloseWorkspace(idx)),
    ]
    .spacing(TAB_ICON_GAP)
    .align_y(alignment::Vertical::Center);

    button(tab_row)
        .on_press(Message::SwitchWorkspace(idx))
        .padding([5, TAB_PAD_H as u16])
        .style(move |_, _| tab_style(active, color))
        .into()
}

fn truncate_name(name: &str, max: usize) -> String {
    if name.chars().count() > max {
        format!("{}...", name.chars().take(max - 3).collect::<String>())
    } else {
        name.to_string()
    }
}

pub fn tab_menu_x_offset(
    workspaces: &[Workspace], renaming: &Option<RenameState>, idx: usize,
) -> f32 {
    let mut x = TAB_ROW_LEFT;
    for (i, ws) in workspaces.iter().enumerate() {
        if i == idx { break; }
        let is_renaming = renaming.as_ref().is_some_and(|r| r.workspace_id == ws.id);
        let text_w = if is_renaming { 120.0 } else {
            ws.name.chars().count().min(20) as f32 * TAB_CHAR_W
        };
        x += text_w + TAB_ICON_GAP + TAB_ICON_W + TAB_ICON_GAP + TAB_ICON_W
            + TAB_PAD_H * 2.0 + TAB_ROW_SPACING;
    }
    x
}
