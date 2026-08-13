use iced::widget::{button, column, container, text};
use iced::{Color, Element, Font, Length};

use crate::message::Message;
use crate::style::*;
use crate::{Toast, ToastKind};

pub const PHOSPHOR_BOLD_BYTES: &[u8] = include_bytes!("../resources/fonts/Phosphor-Bold.ttf");
pub const PHOSPHOR: Font = Font::with_name("Phosphor-Bold");

pub const ICO_MINUS: char = '\u{E32A}';
pub const ICO_SQUARE: char = '\u{E45E}';
pub const ICO_X: char = '\u{E4F6}';
pub const ICO_PLUS: char = '\u{E3D4}';
pub const ICO_DOTS_THREE: char = '\u{E208}';
pub const ICO_CARET_RIGHT: char = '\u{E13A}';

pub fn dialog_surface_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(BG_SURFACE)),
        border: iced::Border {
            color: BORDER_SOFT,
            width: 1.0,
            radius: 14.0.into(),
        },
        shadow: iced::Shadow {
            color: SHADOW_SOFT,
            offset: iced::Vector::new(0.0, 18.0),
            blur_radius: 42.0,
        },
        ..Default::default()
    }
}

pub fn menu_surface_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(BG_SURFACE)),
        border: iced::Border {
            color: BORDER_SOFT,
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: iced::Shadow {
            color: SHADOW_SOFT,
            offset: iced::Vector::new(0.0, 14.0),
            blur_radius: 32.0,
        },
        ..Default::default()
    }
}

pub fn tooltip_surface_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(BG_SURFACE_ALT)),
        border: iced::Border {
            color: BORDER_STRONG,
            width: 1.0,
            radius: 10.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.12),
            offset: iced::Vector::new(0.0, 10.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    }
}

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

pub fn view_toasts<'a>(toasts: &'a [Toast]) -> Element<'a, Message> {
    let items = toasts.iter().map(|toast| {
        let (accent, bg) = match toast.kind {
            ToastKind::Info => (ACCENT, BG_SURFACE),
            ToastKind::Success => (Color::from_rgb(0.596, 0.765, 0.475), BG_SURFACE),
            ToastKind::Error => (Color::from_rgb(0.878, 0.424, 0.459), BG_SURFACE),
        };

        container(
            text(&toast.message)
                .size(12)
                .color(FG_ACTIVE)
        )
        .padding([11, 13])
        .width(Length::Fixed(336.0))
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                color: accent,
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: iced::Shadow {
                color: SHADOW_SOFT,
                offset: iced::Vector::new(0.0, 14.0),
                blur_radius: 30.0,
            },
            ..Default::default()
        })
        .into()
    });

    container(column(items).spacing(8))
        .padding(iced::Padding {
            right: 18.0,
            bottom: 18.0,
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Right)
        .align_y(iced::alignment::Vertical::Bottom)
        .into()
}
