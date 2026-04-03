use iced::widget::{button, text_input};
use iced::{Color, Theme};

pub const BG_TITLEBAR: Color = Color::from_rgb(0.07, 0.07, 0.07);
pub const BG_TERMINAL: Color = Color::from_rgb(0.094, 0.094, 0.094);
pub const FG_ACTIVE: Color = Color::WHITE;
pub const FG_INACTIVE: Color = Color::from_rgb(0.55, 0.55, 0.55);
pub const FG_DIM: Color = Color::from_rgb(0.4, 0.4, 0.4);
pub const FG_MUTED: Color = Color::from_rgb(0.25, 0.25, 0.25);
pub const BG_MENU: Color = Color::from_rgb(0.12, 0.12, 0.12);
pub const BG_MENU_HOVER: Color = Color::from_rgb(0.18, 0.18, 0.18);
pub const BG_HOVER_SUBTLE: Color = Color::from_rgb(0.3, 0.3, 0.3);
pub const TITLEBAR_H: f32 = 36.0;

pub const TAB_PAD_H: f32 = 10.0;
pub const TAB_ICON_W: f32 = 18.0;
pub const TAB_ICON_GAP: f32 = 4.0;
pub const TAB_ROW_LEFT: f32 = 8.0;
pub const TAB_ROW_SPACING: f32 = 2.0;
pub const TAB_CHAR_W: f32 = 6.5;

pub const TAB_COLORS: &[(Color, &str)] = &[
    (Color::from_rgb(0.90, 0.30, 0.30), "Red"),
    (Color::from_rgb(0.95, 0.55, 0.25), "Orange"),
    (Color::from_rgb(0.90, 0.80, 0.25), "Yellow"),
    (Color::from_rgb(0.35, 0.75, 0.40), "Green"),
    (Color::from_rgb(0.30, 0.65, 0.90), "Blue"),
    (Color::from_rgb(0.55, 0.40, 0.85), "Purple"),
    (Color::from_rgb(0.85, 0.40, 0.70), "Pink"),
    (Color::from_rgb(0.45, 0.75, 0.75), "Teal"),
];

pub fn tab_style(active: bool, tab_color: Option<Color>) -> button::Style {
    let (bg, fg) = if active {
        match tab_color {
            Some(c) => {
                let bg = Color::from_rgb(
                    BG_TERMINAL.r * 0.7 + c.r * 0.3,
                    BG_TERMINAL.g * 0.7 + c.g * 0.3,
                    BG_TERMINAL.b * 0.7 + c.b * 0.3,
                );
                (bg, FG_ACTIVE)
            }
            None => (BG_TERMINAL, FG_ACTIVE),
        }
    } else {
        (BG_TITLEBAR, FG_INACTIVE)
    };

    let (border_color, border_width) = match tab_color {
        Some(c) => (c, 2.0),
        None => (Color::TRANSPARENT, 0.0),
    };

    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: fg,
        border: iced::Border {
            color: border_color,
            width: border_width,
            radius: iced::border::Radius::new(4.0).bottom(0.0),
        },
        ..Default::default()
    }
}

pub fn rename_input_style(_: &Theme, _: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1)),
        border: iced::Border {
            color: BG_HOVER_SUBTLE,
            width: 1.0,
            radius: 3.0.into(),
        },
        icon: FG_DIM,
        placeholder: FG_DIM,
        value: FG_ACTIVE,
        selection: Color::from_rgb(0.3, 0.3, 0.5),
    }
}
