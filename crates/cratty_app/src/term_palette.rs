use iced::Color;

use crate::term_colors::ANSI_COLORS;

/// Resolve a full xterm 256-color index to an iced Color.
///
/// - 0..15: standard ANSI colors
/// - 16..231: 6x6x6 color cube
/// - 232..255: grayscale ramp
pub fn indexed_color(idx: u8) -> Color {
    match idx {
        0..=15 => ANSI_COLORS[idx as usize],
        16..=231 => cube_color(idx - 16),
        232..=255 => grayscale_color(idx - 232),
    }
}

/// 6x6x6 color cube. Index 0..215 maps to (r,g,b) each in 0..6.
/// Component value: 0 => 0, 1..5 => component * 40 + 55.
fn cube_color(idx: u8) -> Color {
    let r_idx = idx / 36;
    let g_idx = (idx % 36) / 6;
    let b_idx = idx % 6;

    let r = cube_component(r_idx);
    let g = cube_component(g_idx);
    let b = cube_component(b_idx);

    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

fn cube_component(c: u8) -> u8 {
    if c == 0 { 0 } else { c * 40 + 55 }
}

/// Grayscale ramp: 24 shades from dark (8) to light (238).
fn grayscale_color(offset: u8) -> Color {
    let v = offset * 10 + 8;
    let f = v as f32 / 255.0;
    Color::from_rgb(f, f, f)
}
