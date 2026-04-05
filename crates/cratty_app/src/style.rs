use iced::Color;

pub const BG_TITLEBAR: Color = Color::from_rgb(0.07, 0.07, 0.07);
pub const BG_TERMINAL: Color = Color::from_rgb(0.094, 0.094, 0.094);
pub const FG_ACTIVE: Color = Color::WHITE;
pub const FG_INACTIVE: Color = Color::from_rgb(0.55, 0.55, 0.55);
pub const FG_DIM: Color = Color::from_rgb(0.4, 0.4, 0.4);
pub const FG_MUTED: Color = Color::from_rgb(0.25, 0.25, 0.25);
pub const BG_MENU_HOVER: Color = Color::from_rgb(0.18, 0.18, 0.18);
pub const TITLEBAR_H: f32 = 36.0;

// Terminal font metrics (monospace approximation from font size)
pub const TERM_FONT_SIZE: f32 = 14.0;
pub const CELL_W: u16 = 8;
pub const CELL_H: u16 = 18;
