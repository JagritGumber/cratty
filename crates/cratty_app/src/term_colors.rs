use alacritty_terminal::vte::ansi::{self, NamedColor};
use iced::Color;

use crate::term_palette;

// Zed One Dark ANSI palette
pub const ANSI_COLORS: [Color; 16] = [
    Color::from_rgb(0.165, 0.153, 0.196),     // Black    #2a2732
    Color::from_rgb(0.878, 0.424, 0.459),     // Red      #e06c75
    Color::from_rgb(0.596, 0.765, 0.475),     // Green    #98c379
    Color::from_rgb(0.898, 0.753, 0.482),     // Yellow   #e5c07b
    Color::from_rgb(0.380, 0.686, 0.937),     // Blue     #61afef
    Color::from_rgb(0.776, 0.471, 0.867),     // Magenta  #c678dd
    Color::from_rgb(0.337, 0.714, 0.761),     // Cyan     #56b6c2
    Color::from_rgb(0.671, 0.698, 0.749),     // White    #abb2bf
    Color::from_rgb(0.388, 0.427, 0.514),     // Bright Black  #636d83
    Color::from_rgb(0.918, 0.522, 0.545),     // Bright Red    #ea858b
    Color::from_rgb(0.667, 0.835, 0.506),     // Bright Green  #aad581
    Color::from_rgb(1.000, 0.847, 0.522),     // Bright Yellow #ffd885
    Color::from_rgb(0.522, 0.757, 1.000),     // Bright Blue   #85c1ff
    Color::from_rgb(0.827, 0.596, 0.922),     // Bright Magenta #d398eb
    Color::from_rgb(0.431, 0.835, 0.871),     // Bright Cyan   #6ed5de
    Color::from_rgb(0.980, 0.980, 0.980),     // Bright White  #fafafa
];

pub const BG: Color = Color::from_rgb(0.165, 0.153, 0.196); // #2a2732
pub const FG: Color = Color::from_rgb(0.686, 0.675, 0.725); // #afacb9

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
