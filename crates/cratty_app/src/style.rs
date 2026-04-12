use iced::{Color, Theme};

// Two-color background model (Cratty Warm Dark)
pub const BG_TITLEBAR: Color = Color::from_rgb(0.200, 0.184, 0.227); // #33303a chrome
pub const BG_TERMINAL: Color = Color::from_rgb(0.165, 0.153, 0.196); // #2a2732 content
pub const ACCENT: Color = Color::from_rgb(0.380, 0.686, 0.937);      // #61afef blue

pub const FG_ACTIVE: Color = Color::from_rgb(0.886, 0.867, 0.906);   // #e2dde7
pub const FG_INACTIVE: Color = Color::from_rgb(0.698, 0.667, 0.741); // #b2aabd
pub const FG_DIM: Color = Color::from_rgb(0.439, 0.400, 0.510);      // #706682
pub const FG_MUTED: Color = Color::from_rgb(0.306, 0.282, 0.353);    // #4e485a

pub const BG_MENU_HOVER: Color = Color::from_rgb(0.227, 0.208, 0.267); // #3a3544
pub const BG_WS_ACTIVE: Color = Color::from_rgb(0.263, 0.243, 0.306); // #433e4e
pub const TITLEBAR_H: f32 = 36.0;

pub fn cratty_theme() -> Theme {
    Theme::custom("Cratty", iced::theme::Palette {
        background: BG_TERMINAL, text: FG_ACTIVE, primary: ACCENT,
        success: Color::from_rgb(0.596, 0.765, 0.475),
        danger: Color::from_rgb(0.878, 0.424, 0.459),
        warning: Color::from_rgb(0.898, 0.753, 0.482),
    })
}
