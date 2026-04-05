use alacritty_terminal::vte::ansi::{self, NamedColor};
use iced::Color;

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

pub fn ansi_to_iced(color: ansi::Color) -> Color {
    match color {
        ansi::Color::Named(n) => match n {
            NamedColor::Background => BG,
            NamedColor::Foreground | NamedColor::Cursor => FG,
            _ => {
                let idx = n as usize;
                if idx < 16 { ANSI_COLORS[idx] } else { FG }
            }
        },
        ansi::Color::Indexed(i) => {
            if (i as usize) < 16 { ANSI_COLORS[i as usize] } else { FG }
        }
        ansi::Color::Spec(rgb) => {
            Color::from_rgb8(rgb.r, rgb.g, rgb.b)
        }
    }
}
