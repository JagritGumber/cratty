use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub foreground: Color,
    pub background: Color,
    pub cursor: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    /// ANSI colors 0-15.
    pub ansi_colors: [Color; 16],
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default-dark".to_string(),
            foreground: Color::new(204, 204, 204),
            background: Color::new(30, 30, 30),
            cursor: Color::new(255, 255, 255),
            selection_bg: Color::new(58, 58, 58),
            selection_fg: Color::new(255, 255, 255),
            ansi_colors: [
                Color::new(0, 0, 0),       // black
                Color::new(205, 49, 49),    // red
                Color::new(13, 188, 121),   // green
                Color::new(229, 229, 16),   // yellow
                Color::new(36, 114, 200),   // blue
                Color::new(188, 63, 188),   // magenta
                Color::new(17, 168, 205),   // cyan
                Color::new(204, 204, 204),  // white
                Color::new(102, 102, 102),  // bright black
                Color::new(241, 76, 76),    // bright red
                Color::new(35, 209, 139),   // bright green
                Color::new(245, 245, 67),   // bright yellow
                Color::new(59, 142, 234),   // bright blue
                Color::new(214, 112, 214),  // bright magenta
                Color::new(41, 184, 219),   // bright cyan
                Color::new(242, 242, 242),  // bright white
            ],
        }
    }
}
