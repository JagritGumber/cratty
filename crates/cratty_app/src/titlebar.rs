use iced::alignment;
use iced::widget::{mouse_area, row, text, Space};
use iced::{Color, Element, Length};

use crate::message::Message;
use crate::style::*;
use crate::widgets::*;

/// Window chrome only: app name, drag area, minimize/maximize/close.
pub fn view_titlebar() -> Element<'static, Message> {
    let label = text("Cratty").size(12).color(FG_DIM);

    let controls = row![
        win_btn(ICO_MINUS, Message::Minimize, FG_DIM, Color::from_rgb(0.2, 0.2, 0.2)),
        win_btn(ICO_SQUARE, Message::Maximize, FG_DIM, Color::from_rgb(0.2, 0.2, 0.2)),
        win_btn(ICO_X, Message::CloseWindow, FG_DIM, Color::from_rgb(0.7, 0.15, 0.15)),
    ]
    .spacing(0);

    let bar = row![
        Space::new().width(12),
        label,
        Space::new().width(Length::Fill),
        controls,
    ]
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
