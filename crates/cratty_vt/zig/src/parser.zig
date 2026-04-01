const std = @import("std");
const Grid = @import("grid.zig").Grid;

/// VT100/VT220/xterm escape sequence parser.
/// Implements a state machine that processes bytes and applies
/// terminal operations to the grid.
pub const Parser = struct {
    state: State = .ground,
    params: [16]u16 = [_]u16{0} ** 16,
    param_count: u8 = 0,
    intermediate: u8 = 0,

    // UTF-8 decoding state.
    utf8_codepoint: u32 = 0,
    utf8_remaining: u8 = 0,

    const State = enum {
        ground,
        escape,
        escape_intermediate,
        csi_entry,
        csi_param,
        csi_intermediate,
        osc_string,
        dcs_entry,
    };

    pub fn init() Parser {
        return .{};
    }

    pub fn feed(self: *Parser, grid: *Grid, data: []const u8) void {
        for (data) |byte| {
            self.process_byte(grid, byte);
        }
    }

    fn process_byte(self: *Parser, grid: *Grid, byte: u8) void {
        // If we're in the middle of a UTF-8 sequence, continue it.
        if (self.utf8_remaining > 0) {
            if (byte & 0xC0 == 0x80) {
                // Valid continuation byte.
                self.utf8_codepoint = (self.utf8_codepoint << 6) | @as(u32, byte & 0x3F);
                self.utf8_remaining -= 1;
                if (self.utf8_remaining == 0) {
                    // Complete codepoint.
                    if (self.utf8_codepoint >= 0x20 and self.state == .ground) {
                        grid.put_char(self.utf8_codepoint);
                    }
                }
                return;
            } else {
                // Invalid continuation — reset and reprocess this byte.
                self.utf8_remaining = 0;
                self.utf8_codepoint = 0;
                grid.put_char(0xFFFD); // replacement character
            }
        }

        // Handle C0 controls in any state.
        switch (byte) {
            0x00...0x06, 0x0E...0x1A, 0x1C...0x1F => return, // Ignored C0 controls
            0x07 => return, // BEL
            0x08 => {
                grid.backspace();
                return;
            },
            0x09 => {
                grid.tab();
                return;
            },
            0x0A, 0x0B, 0x0C => {
                grid.line_feed();
                return;
            },
            0x0D => {
                grid.carriage_return();
                return;
            },
            0x1B => {
                self.state = .escape;
                self.reset_params();
                return;
            },
            else => {},
        }

        switch (self.state) {
            .ground => self.ground_state(grid, byte),
            .escape => self.escape_state(grid, byte),
            .escape_intermediate => self.escape_intermediate_state(grid, byte),
            .csi_entry => self.csi_entry_state(grid, byte),
            .csi_param => self.csi_param_state(grid, byte),
            .csi_intermediate => self.csi_intermediate_state(grid, byte),
            .osc_string => self.osc_state(byte),
            .dcs_entry => self.dcs_state(byte),
        }
    }

    fn ground_state(self: *Parser, grid: *Grid, byte: u8) void {
        if (byte >= 0x20 and byte < 0x7F) {
            // Printable ASCII.
            grid.put_char(@as(u32, byte));
        } else if (byte >= 0xC0 and byte < 0xE0) {
            // 2-byte UTF-8 sequence.
            self.utf8_codepoint = @as(u32, byte & 0x1F);
            self.utf8_remaining = 1;
        } else if (byte >= 0xE0 and byte < 0xF0) {
            // 3-byte UTF-8 sequence.
            self.utf8_codepoint = @as(u32, byte & 0x0F);
            self.utf8_remaining = 2;
        } else if (byte >= 0xF0 and byte < 0xF8) {
            // 4-byte UTF-8 sequence.
            self.utf8_codepoint = @as(u32, byte & 0x07);
            self.utf8_remaining = 3;
        } else if (byte == 0x7F) {
            // DEL — ignore.
        }
    }

    fn escape_state(self: *Parser, grid: *Grid, byte: u8) void {
        switch (byte) {
            '[' => {
                self.state = .csi_entry;
                self.reset_params();
            },
            ']' => {
                self.state = .osc_string;
            },
            'P' => {
                self.state = .dcs_entry;
            },
            '(', ')' => {
                self.state = .escape_intermediate;
                self.intermediate = byte;
            },
            'M' => {
                // Reverse index.
                if (grid.cursor.row > 0) {
                    grid.cursor.row -= 1;
                } else {
                    grid.scroll_down();
                }
                self.state = .ground;
            },
            'D' => {
                grid.line_feed();
                self.state = .ground;
            },
            'E' => {
                grid.carriage_return();
                grid.line_feed();
                self.state = .ground;
            },
            'c' => {
                grid.clear_screen();
                grid.pen_fg = .{ 204, 204, 204 };
                grid.pen_bg = .{ 30, 30, 30 };
                grid.pen_attrs = .{};
                self.state = .ground;
            },
            '7' => {
                grid.save_cursor();
                self.state = .ground;
            },
            '8' => {
                grid.restore_cursor();
                self.state = .ground;
            },
            '=' => {
                // Application keypad mode — ignore.
                self.state = .ground;
            },
            '>' => {
                // Normal keypad mode — ignore.
                self.state = .ground;
            },
            else => {
                self.state = .ground;
            },
        }
    }

    fn escape_intermediate_state(self: *Parser, _: *Grid, _: u8) void {
        self.state = .ground;
    }

    fn csi_entry_state(self: *Parser, grid: *Grid, byte: u8) void {
        if (byte >= '0' and byte <= '9') {
            self.params[0] = byte - '0';
            self.param_count = 1;
            self.state = .csi_param;
        } else if (byte == ';') {
            self.param_count = 2;
            self.state = .csi_param;
        } else if (byte == '?') {
            self.intermediate = byte;
            self.state = .csi_param;
        } else if (byte == '>') {
            self.intermediate = byte;
            self.state = .csi_param;
        } else if (byte >= 0x40 and byte <= 0x7E) {
            self.param_count = 0;
            self.dispatch_csi(grid, byte);
        } else {
            self.state = .ground;
        }
    }

    fn csi_param_state(self: *Parser, grid: *Grid, byte: u8) void {
        if (byte >= '0' and byte <= '9') {
            if (self.param_count == 0) self.param_count = 1;
            const idx = self.param_count - 1;
            if (idx < self.params.len) {
                self.params[idx] = self.params[idx] *% 10 +% (byte - '0');
            }
        } else if (byte == ';') {
            if (self.param_count < self.params.len) {
                self.param_count += 1;
            }
        } else if (byte >= 0x40 and byte <= 0x7E) {
            self.dispatch_csi(grid, byte);
        } else if (byte >= 0x20 and byte < 0x40) {
            self.intermediate = byte;
            self.state = .csi_intermediate;
        } else {
            self.state = .ground;
        }
    }

    fn csi_intermediate_state(self: *Parser, grid: *Grid, byte: u8) void {
        if (byte >= 0x40 and byte <= 0x7E) {
            self.dispatch_csi(grid, byte);
        } else if (byte < 0x20 or byte >= 0x7F) {
            self.state = .ground;
        }
    }

    fn osc_state(self: *Parser, byte: u8) void {
        if (byte == 0x07) {
            self.state = .ground;
        } else if (byte == 0x1B) {
            self.state = .escape;
        }
    }

    fn dcs_state(self: *Parser, byte: u8) void {
        if (byte == 0x1B) {
            self.state = .escape;
        }
    }

    fn dispatch_csi(self: *Parser, grid: *Grid, final_byte: u8) void {
        const p0 = if (self.param_count >= 1) self.params[0] else 0;
        const p1 = if (self.param_count >= 2) self.params[1] else 0;

        if (self.intermediate == '?') {
            self.dispatch_private_mode(grid, final_byte, p0);
            self.state = .ground;
            return;
        }

        if (self.intermediate == '>' or self.intermediate == ' ') {
            // DA2 response, DECSCUSR, etc. — ignore for now.
            self.state = .ground;
            return;
        }

        switch (final_byte) {
            'A' => {
                const n: u16 = if (p0 > 0) p0 else 1;
                grid.cursor.row -|= n;
            },
            'B' => {
                const n: u16 = if (p0 > 0) p0 else 1;
                grid.cursor.row = @min(grid.cursor.row + n, grid.rows -| 1);
            },
            'C' => {
                const n: u16 = if (p0 > 0) p0 else 1;
                grid.cursor.col = @min(grid.cursor.col + n, grid.cols -| 1);
            },
            'D' => {
                const n: u16 = if (p0 > 0) p0 else 1;
                grid.cursor.col -|= n;
            },
            'E' => {
                const n: u16 = if (p0 > 0) p0 else 1;
                grid.cursor.row = @min(grid.cursor.row + n, grid.rows -| 1);
                grid.cursor.col = 0;
            },
            'F' => {
                const n: u16 = if (p0 > 0) p0 else 1;
                grid.cursor.row -|= n;
                grid.cursor.col = 0;
            },
            'G' => {
                const col: u16 = if (p0 > 0) p0 - 1 else 0;
                grid.cursor.col = @min(col, grid.cols -| 1);
            },
            'H', 'f' => {
                const row: u16 = if (p0 > 0) p0 - 1 else 0;
                const col: u16 = if (p1 > 0) p1 - 1 else 0;
                grid.set_cursor(col, row);
            },
            'J' => {
                switch (p0) {
                    0 => grid.clear_to_eos(),
                    1 => grid.clear_to_bos(),
                    2, 3 => grid.clear_screen(),
                    else => {},
                }
            },
            'K' => {
                switch (p0) {
                    0 => grid.clear_to_eol(),
                    1 => grid.clear_to_bol(),
                    2 => grid.clear_line(),
                    else => {},
                }
            },
            'L' => {
                const n: u16 = if (p0 > 0) p0 else 1;
                grid.insert_lines(n);
            },
            'M' => {
                const n: u16 = if (p0 > 0) p0 else 1;
                grid.delete_lines(n);
            },
            'P' => {
                const n: u16 = if (p0 > 0) p0 else 1;
                grid.delete_chars(n);
            },
            '@' => {
                const n: u16 = if (p0 > 0) p0 else 1;
                grid.insert_chars(n);
            },
            'X' => {
                const n: u16 = if (p0 > 0) p0 else 1;
                grid.erase_chars(n);
            },
            'S' => {
                const n: u16 = if (p0 > 0) p0 else 1;
                var i: u16 = 0;
                while (i < n) : (i += 1) {
                    grid.scroll_up();
                }
            },
            'T' => {
                const n: u16 = if (p0 > 0) p0 else 1;
                var i: u16 = 0;
                while (i < n) : (i += 1) {
                    grid.scroll_down();
                }
            },
            'd' => {
                const row: u16 = if (p0 > 0) p0 - 1 else 0;
                grid.cursor.row = @min(row, grid.rows -| 1);
            },
            'm' => {
                self.dispatch_sgr(grid);
            },
            'r' => {
                // Set Scrolling Region.
                const top: u16 = if (p0 > 0) p0 - 1 else 0;
                const bottom: u16 = if (p1 > 0) p1 else grid.rows;
                grid.set_scroll_region(top, bottom);
                grid.set_cursor(0, 0);
            },
            's' => {
                grid.save_cursor();
            },
            'u' => {
                grid.restore_cursor();
            },
            'h' => {
                // Set mode — handle auto-wrap.
                if (p0 == 4) grid.insert_mode = true;
            },
            'l' => {
                if (p0 == 4) grid.insert_mode = false;
            },
            'n' => {
                // Device Status Report — we can't respond from here.
                // The PTY layer would need to handle this.
            },
            'c' => {
                // Device Attributes — ignore.
            },
            't' => {
                // Window manipulation — ignore.
            },
            else => {},
        }
        self.state = .ground;
    }

    fn dispatch_private_mode(self: *Parser, grid: *Grid, final_byte: u8, mode: u16) void {
        switch (final_byte) {
            'h' => {
                // Set private mode.
                switch (mode) {
                    1 => grid.app_cursor_keys = true,
                    7 => grid.auto_wrap = true,
                    12 => {}, // Start blinking cursor — ignore
                    25 => grid.cursor_visible = true,
                    1049 => grid.enter_alt_screen(),
                    1004 => {}, // Focus events — ignore
                    2004 => grid.bracketed_paste = true,
                    else => {},
                }
            },
            'l' => {
                // Reset private mode.
                switch (mode) {
                    1 => grid.app_cursor_keys = false,
                    7 => grid.auto_wrap = false,
                    12 => {},
                    25 => grid.cursor_visible = false,
                    1049 => grid.leave_alt_screen(),
                    1004 => {},
                    2004 => grid.bracketed_paste = false,
                    else => {},
                }
            },
            else => {},
        }
        _ = self;
    }

    fn dispatch_sgr(self: *Parser, grid: *Grid) void {
        if (self.param_count == 0) {
            grid.pen_fg = .{ 204, 204, 204 };
            grid.pen_bg = .{ 30, 30, 30 };
            grid.pen_attrs = .{};
            return;
        }

        var i: u8 = 0;
        while (i < self.param_count) : (i += 1) {
            const p = self.params[i];
            switch (p) {
                0 => {
                    grid.pen_fg = .{ 204, 204, 204 };
                    grid.pen_bg = .{ 30, 30, 30 };
                    grid.pen_attrs = .{};
                },
                1 => grid.pen_attrs.bold = true,
                2 => grid.pen_attrs.dim = true,
                3 => grid.pen_attrs.italic = true,
                4 => grid.pen_attrs.underline = true,
                7 => grid.pen_attrs.inverse = true,
                9 => grid.pen_attrs.strikethrough = true,
                22 => {
                    grid.pen_attrs.bold = false;
                    grid.pen_attrs.dim = false;
                },
                23 => grid.pen_attrs.italic = false,
                24 => grid.pen_attrs.underline = false,
                27 => grid.pen_attrs.inverse = false,
                29 => grid.pen_attrs.strikethrough = false,

                30...37 => {
                    const colors = ansi_color_table();
                    grid.pen_fg = colors[p - 30];
                },
                39 => grid.pen_fg = .{ 204, 204, 204 },

                40...47 => {
                    const colors = ansi_color_table();
                    grid.pen_bg = colors[p - 40];
                },
                49 => grid.pen_bg = .{ 30, 30, 30 },

                90...97 => {
                    const colors = ansi_color_table();
                    grid.pen_fg = colors[p - 90 + 8];
                },

                100...107 => {
                    const colors = ansi_color_table();
                    grid.pen_bg = colors[p - 100 + 8];
                },

                38 => {
                    if (i + 1 < self.param_count and self.params[i + 1] == 5) {
                        if (i + 2 < self.param_count) {
                            grid.pen_fg = color_256(self.params[i + 2]);
                            i += 2;
                        }
                    } else if (i + 1 < self.param_count and self.params[i + 1] == 2) {
                        if (i + 4 < self.param_count) {
                            grid.pen_fg[0] = @truncate(self.params[i + 2]);
                            grid.pen_fg[1] = @truncate(self.params[i + 3]);
                            grid.pen_fg[2] = @truncate(self.params[i + 4]);
                            i += 4;
                        }
                    }
                },
                48 => {
                    if (i + 1 < self.param_count and self.params[i + 1] == 5) {
                        if (i + 2 < self.param_count) {
                            grid.pen_bg = color_256(self.params[i + 2]);
                            i += 2;
                        }
                    } else if (i + 1 < self.param_count and self.params[i + 1] == 2) {
                        if (i + 4 < self.param_count) {
                            grid.pen_bg[0] = @truncate(self.params[i + 2]);
                            grid.pen_bg[1] = @truncate(self.params[i + 3]);
                            grid.pen_bg[2] = @truncate(self.params[i + 4]);
                            i += 4;
                        }
                    }
                },

                else => {},
            }
        }
    }

    fn reset_params(self: *Parser) void {
        self.params = [_]u16{0} ** 16;
        self.param_count = 0;
        self.intermediate = 0;
    }
};

fn ansi_color_table() [16][3]u8 {
    return .{
        .{ 0, 0, 0 },
        .{ 205, 49, 49 },
        .{ 13, 188, 121 },
        .{ 229, 229, 16 },
        .{ 36, 114, 200 },
        .{ 188, 63, 188 },
        .{ 17, 168, 205 },
        .{ 204, 204, 204 },
        .{ 102, 102, 102 },
        .{ 241, 76, 76 },
        .{ 35, 209, 139 },
        .{ 245, 245, 67 },
        .{ 59, 142, 234 },
        .{ 214, 112, 214 },
        .{ 41, 184, 219 },
        .{ 242, 242, 242 },
    };
}

fn color_256(idx: u16) [3]u8 {
    if (idx < 16) {
        const table = ansi_color_table();
        return table[idx];
    } else if (idx < 232) {
        const ci = idx - 16;
        const r_idx = ci / 36;
        const g_idx = (ci % 36) / 6;
        const b_idx = ci % 6;
        const r: u8 = if (r_idx > 0) @truncate(r_idx * 40 + 55) else 0;
        const g: u8 = if (g_idx > 0) @truncate(g_idx * 40 + 55) else 0;
        const b: u8 = if (b_idx > 0) @truncate(b_idx * 40 + 55) else 0;
        return .{ r, g, b };
    } else {
        const level: u8 = @truncate((idx - 232) * 10 + 8);
        return .{ level, level, level };
    }
}
