use iced::widget::{button, container, text};
use iced::{Color, Element, Font, Length};

use crate::message::Message;
use crate::style::*;

pub const PHOSPHOR_BOLD_BYTES: &[u8] = include_bytes!("../resources/fonts/Phosphor-Bold.ttf");
pub const PHOSPHOR: Font = Font::with_name("Phosphor-Bold");

pub const ICO_MINUS: char = '\u{E32A}';
pub const ICO_SQUARE: char = '\u{E45E}';
pub const ICO_X: char = '\u{E4F6}';
pub const ICO_PLUS: char = '\u{E3D4}';
pub const ICO_DOTS_THREE: char = '\u{E208}';

pub fn centered_icon(codepoint: char, size: f32) -> Element<'static, Message> {
    container(
        text(codepoint)
            .font(PHOSPHOR)
            .size(size)
            .shaping(text::Shaping::Advanced),
    )
    .center(Length::Fill)
    .into()
}

pub fn win_btn(
    icon: char, msg: Message, fg: Color, hover_bg: Color,
) -> Element<'static, Message> {
    button(centered_icon(icon, 14.0))
        .on_press(msg)
        .width(46)
        .height(TITLEBAR_H)
        .padding(0)
        .style(move |_, status| button::Style {
            background: match status {
                button::Status::Hovered => Some(iced::Background::Color(hover_bg)),
                _ => None,
            },
            text_color: match status {
                button::Status::Hovered => FG_ACTIVE,
                _ => fg,
            },
            ..Default::default()
        })
        .into()
}
