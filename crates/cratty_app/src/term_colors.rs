use alacritty_terminal::vte::ansi::{self, NamedColor};
use iced::Color;

use crate::term_palette;

pub const ANSI_COLORS: [Color; 16] = [
    Color::from_rgb(0.0, 0.0, 0.0),           // Black
    Color::from_rgb(0.8, 0.0, 0.0),           // Red
    Color::from_rgb(0.0, 0.8, 0.0),           // Green
    Color::from_rgb(0.8, 0.8, 0.0),           // Yellow
    Color::from_rgb(0.0, 0.0, 0.8),           // Blue
    Color::from_rgb(0.8, 0.0, 0.8),           // Magenta
    Color::from_rgb(0.0, 0.8, 0.8),           // Cyan
    Color::from_rgb(0.75, 0.75, 0.75),        // White
    Color::from_rgb(0.5, 0.5, 0.5),           // Bright Black
    Color::from_rgb(1.0, 0.0, 0.0),           // Bright Red
    Color::from_rgb(0.0, 1.0, 0.0),           // Bright Green
    Color::from_rgb(1.0, 1.0, 0.0),           // Bright Yellow
    Color::from_rgb(0.3, 0.5, 1.0),           // Bright Blue
    Color::from_rgb(1.0, 0.0, 1.0),           // Bright Magenta
    Color::from_rgb(0.0, 1.0, 1.0),           // Bright Cyan
    Color::WHITE,                              // Bright White
];

pub const BG: Color = Color::from_rgb(0.094, 0.094, 0.094);
pub const FG: Color = Color::from_rgb(0.8, 0.8, 0.8);

/// Map an alacritty color to an iced Color.
/// When `is_bold` is true, named/indexed colors 0..7 promote to their bright variant.
pub fn ansi_to_iced(color: ansi::Color, is_bold: bool) -> Color {
    match color {
        ansi::Color::Named(n) => match n {
            NamedColor::Background => BG,
            NamedColor::Foreground | NamedColor::Cursor => FG,
            _ => {
                let idx = n as u8;
                let idx = if is_bold && idx < 8 { idx + 8 } else { idx };
                term_palette::indexed_color(idx)
            }
        },
        ansi::Color::Indexed(i) => {
            let i = if is_bold && i < 8 { i + 8 } else { i };
            term_palette::indexed_color(i)
        }
        ansi::Color::Spec(rgb) => Color::from_rgb8(rgb.r, rgb.g, rgb.b),
    }
}
