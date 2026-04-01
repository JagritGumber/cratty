/// A single terminal cell. Packed for cache efficiency.
/// Each cell is 16 bytes: 4 bytes char + 3 fg + 3 bg + 2 attrs + 4 padding.
pub const Cell = extern struct {
    /// Unicode codepoint (0 = empty).
    char: u32 = 0,

    /// Foreground color RGB.
    fg_r: u8 = 204,
    fg_g: u8 = 204,
    fg_b: u8 = 204,

    /// Background color RGB.
    bg_r: u8 = 30,
    bg_g: u8 = 30,
    bg_b: u8 = 30,

    /// Attribute flags.
    attrs: Attrs = .{},

    /// Reserved for wide char / combining char info.
    _pad: u8 = 0,

    pub const Attrs = packed struct(u8) {
        bold: bool = false,
        italic: bool = false,
        underline: bool = false,
        strikethrough: bool = false,
        inverse: bool = false,
        dim: bool = false,
        _reserved: u2 = 0,
    };

    pub fn clear(self: *Cell) void {
        self.* = .{};
    }

    pub fn set_char(self: *Cell, c: u32) void {
        self.char = c;
    }

    pub fn set_fg(self: *Cell, r: u8, g: u8, b: u8) void {
        self.fg_r = r;
        self.fg_g = g;
        self.fg_b = b;
    }

    pub fn set_bg(self: *Cell, r: u8, g: u8, b: u8) void {
        self.bg_r = r;
        self.bg_g = g;
        self.bg_b = b;
    }
};
