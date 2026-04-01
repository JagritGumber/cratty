const std = @import("std");
const Cell = @import("cell.zig").Cell;
const Allocator = std.mem.Allocator;

pub const Cursor = struct {
    col: u16 = 0,
    row: u16 = 0,
};

pub const Grid = struct {
    cols: u16,
    rows: u16,
    cells: []Cell,
    scrollback: std.ArrayListUnmanaged([]Cell) = .empty,
    scrollback_limit: u32,
    viewport_offset: u32 = 0,
    cursor: Cursor = .{},
    saved_cursor: Cursor = .{},
    allocator: Allocator,

    // SGR pen state.
    pen_fg: [3]u8 = .{ 204, 204, 204 },
    pen_bg: [3]u8 = .{ 30, 30, 30 },
    pen_attrs: Cell.Attrs = .{},

    // Scroll region (0-indexed, exclusive end).
    scroll_top: u16 = 0,
    scroll_bottom: u16 = 0, // 0 means "rows" (full screen)

    // Mode flags.
    auto_wrap: bool = true,
    insert_mode: bool = false,
    cursor_visible: bool = true,
    app_cursor_keys: bool = false,
    bracketed_paste: bool = false,

    // Alternate screen buffer.
    alt_cells: ?[]Cell = null,
    alt_cursor: Cursor = .{},
    is_alt_screen: bool = false,

    pub fn init(allocator: Allocator, cols: u16, rows: u16, scrollback_limit: u32) !Grid {
        const total = @as(usize, cols) * @as(usize, rows);
        const cells = try allocator.alloc(Cell, total);
        @memset(cells, Cell{});

        return Grid{
            .cols = cols,
            .rows = rows,
            .cells = cells,
            .scrollback_limit = scrollback_limit,
            .scroll_bottom = rows,
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *Grid) void {
        self.allocator.free(self.cells);
        for (self.scrollback.items) |line| {
            self.allocator.free(line);
        }
        self.scrollback.deinit(self.allocator);
        if (self.alt_cells) |ac| {
            self.allocator.free(ac);
        }
    }

    fn effective_scroll_bottom(self: *const Grid) u16 {
        return if (self.scroll_bottom == 0) self.rows else self.scroll_bottom;
    }

    pub fn get_cell(self: *const Grid, col: u16, row: u16) ?*const Cell {
        if (col >= self.cols or row >= self.rows) return null;
        const idx = @as(usize, row) * @as(usize, self.cols) + @as(usize, col);
        return &self.cells[idx];
    }

    pub fn get_cell_mut(self: *Grid, col: u16, row: u16) ?*Cell {
        if (col >= self.cols or row >= self.rows) return null;
        const idx = @as(usize, row) * @as(usize, self.cols) + @as(usize, col);
        return &self.cells[idx];
    }

    pub fn put_char(self: *Grid, c: u32) void {
        if (self.cursor.col >= self.cols) {
            if (self.auto_wrap) {
                self.carriage_return();
                self.line_feed();
            } else {
                self.cursor.col = self.cols - 1;
            }
        }

        if (self.insert_mode) {
            self.insert_chars(1);
        }

        if (self.get_cell_mut(self.cursor.col, self.cursor.row)) |cell| {
            cell.char = c;
            cell.fg_r = self.pen_fg[0];
            cell.fg_g = self.pen_fg[1];
            cell.fg_b = self.pen_fg[2];
            cell.bg_r = self.pen_bg[0];
            cell.bg_g = self.pen_bg[1];
            cell.bg_b = self.pen_bg[2];
            cell.attrs = self.pen_attrs;
        }

        self.cursor.col += 1;
    }

    pub fn carriage_return(self: *Grid) void {
        self.cursor.col = 0;
    }

    pub fn line_feed(self: *Grid) void {
        const bottom = self.effective_scroll_bottom();
        if (self.cursor.row + 1 >= bottom) {
            self.scroll_region_up(self.scroll_top, bottom);
        } else {
            self.cursor.row += 1;
        }
    }

    pub fn backspace(self: *Grid) void {
        if (self.cursor.col > 0) {
            self.cursor.col -= 1;
        }
    }

    pub fn tab(self: *Grid) void {
        const next = (self.cursor.col / 8 + 1) * 8;
        self.cursor.col = @min(next, self.cols -| 1);
    }

    // --- Scroll operations ---

    pub fn scroll_up(self: *Grid) void {
        const bottom = self.effective_scroll_bottom();
        self.scroll_region_up(self.scroll_top, bottom);
    }

    pub fn scroll_down(self: *Grid) void {
        const bottom = self.effective_scroll_bottom();
        self.scroll_region_down(self.scroll_top, bottom);
    }

    fn scroll_region_up(self: *Grid, top: u16, bottom: u16) void {
        if (top >= bottom or top >= self.rows) return;

        // Save top line to scrollback (only if scrolling the whole screen).
        if (top == 0 and bottom == self.rows and !self.is_alt_screen) {
            const line = self.allocator.alloc(Cell, self.cols) catch return;
            const top_end: usize = @as(usize, self.cols);
            @memcpy(line, self.cells[0..top_end]);
            self.scrollback.append(self.allocator, line) catch {
                self.allocator.free(line);
            };
            while (self.scrollback.items.len > self.scrollback_limit) {
                const old = self.scrollback.orderedRemove(0);
                self.allocator.free(old);
            }
        }

        // Shift rows up within the region.
        const cols: usize = @as(usize, self.cols);
        var row = top;
        while (row + 1 < bottom) : (row += 1) {
            const dst_start = @as(usize, row) * cols;
            const src_start = @as(usize, row + 1) * cols;
            @memmove(self.cells[dst_start .. dst_start + cols], self.cells[src_start .. src_start + cols]);
        }

        // Clear bottom row of region.
        const clear_start = @as(usize, bottom - 1) * cols;
        @memset(self.cells[clear_start .. clear_start + cols], Cell{});
    }

    fn scroll_region_down(self: *Grid, top: u16, bottom: u16) void {
        if (top >= bottom or top >= self.rows) return;

        const cols: usize = @as(usize, self.cols);
        var row = bottom - 1;
        while (row > top) : (row -= 1) {
            const dst_start = @as(usize, row) * cols;
            const src_start = @as(usize, row - 1) * cols;
            @memmove(self.cells[dst_start .. dst_start + cols], self.cells[src_start .. src_start + cols]);
        }

        // Clear top row of region.
        const clear_start = @as(usize, top) * cols;
        @memset(self.cells[clear_start .. clear_start + cols], Cell{});
    }

    pub fn set_scroll_region(self: *Grid, top: u16, bottom: u16) void {
        self.scroll_top = @min(top, self.rows -| 1);
        self.scroll_bottom = @min(bottom, self.rows);
        if (self.scroll_top >= self.scroll_bottom) {
            self.scroll_top = 0;
            self.scroll_bottom = self.rows;
        }
    }

    // --- Clear operations ---

    pub fn clear_screen(self: *Grid) void {
        @memset(self.cells, Cell{});
        self.cursor = .{};
    }

    pub fn clear_to_eol(self: *Grid) void {
        var col = self.cursor.col;
        while (col < self.cols) : (col += 1) {
            if (self.get_cell_mut(col, self.cursor.row)) |c| c.clear();
        }
    }

    pub fn clear_to_bol(self: *Grid) void {
        var col: u16 = 0;
        while (col <= self.cursor.col and col < self.cols) : (col += 1) {
            if (self.get_cell_mut(col, self.cursor.row)) |c| c.clear();
        }
    }

    pub fn clear_line(self: *Grid) void {
        var col: u16 = 0;
        while (col < self.cols) : (col += 1) {
            if (self.get_cell_mut(col, self.cursor.row)) |c| c.clear();
        }
    }

    pub fn clear_to_eos(self: *Grid) void {
        self.clear_to_eol();
        var row = self.cursor.row + 1;
        while (row < self.rows) : (row += 1) {
            var col: u16 = 0;
            while (col < self.cols) : (col += 1) {
                if (self.get_cell_mut(col, row)) |c| c.clear();
            }
        }
    }

    pub fn clear_to_bos(self: *Grid) void {
        // Clear from start of screen to cursor.
        var row: u16 = 0;
        while (row < self.cursor.row) : (row += 1) {
            var col: u16 = 0;
            while (col < self.cols) : (col += 1) {
                if (self.get_cell_mut(col, row)) |c| c.clear();
            }
        }
        self.clear_to_bol();
    }

    pub fn erase_chars(self: *Grid, n: u16) void {
        var col = self.cursor.col;
        const end = @min(self.cursor.col + n, self.cols);
        while (col < end) : (col += 1) {
            if (self.get_cell_mut(col, self.cursor.row)) |c| c.clear();
        }
    }

    // --- Insert/delete operations ---

    pub fn insert_chars(self: *Grid, n: u16) void {
        const row_off = @as(usize, self.cursor.row) * @as(usize, self.cols);
        const start = @as(usize, self.cursor.col);
        const count = @as(usize, @min(n, self.cols -| self.cursor.col));
        const end = @as(usize, self.cols);

        // Shift right.
        if (start + count < end) {
            var i = end - 1;
            while (i >= start + count) : (i -= 1) {
                self.cells[row_off + i] = self.cells[row_off + i - count];
                if (i == start + count) break;
            }
        }

        // Clear inserted cells.
        var j: usize = start;
        while (j < start + count and j < end) : (j += 1) {
            self.cells[row_off + j] = Cell{};
        }
    }

    pub fn delete_chars(self: *Grid, n: u16) void {
        const row_off = @as(usize, self.cursor.row) * @as(usize, self.cols);
        const start = @as(usize, self.cursor.col);
        const count = @as(usize, @min(n, self.cols -| self.cursor.col));
        const end = @as(usize, self.cols);

        // Shift left.
        var i = start;
        while (i + count < end) : (i += 1) {
            self.cells[row_off + i] = self.cells[row_off + i + count];
        }

        // Clear vacated cells.
        while (i < end) : (i += 1) {
            self.cells[row_off + i] = Cell{};
        }
    }

    pub fn insert_lines(self: *Grid, n: u16) void {
        const bottom = self.effective_scroll_bottom();
        if (self.cursor.row < self.scroll_top or self.cursor.row >= bottom) return;
        var i: u16 = 0;
        while (i < n) : (i += 1) {
            self.scroll_region_down(self.cursor.row, bottom);
        }
    }

    pub fn delete_lines(self: *Grid, n: u16) void {
        const bottom = self.effective_scroll_bottom();
        if (self.cursor.row < self.scroll_top or self.cursor.row >= bottom) return;
        var i: u16 = 0;
        while (i < n) : (i += 1) {
            self.scroll_region_up(self.cursor.row, bottom);
        }
    }

    // --- Cursor save/restore ---

    pub fn save_cursor(self: *Grid) void {
        self.saved_cursor = self.cursor;
    }

    pub fn restore_cursor(self: *Grid) void {
        self.cursor = self.saved_cursor;
        self.cursor.col = @min(self.cursor.col, self.cols -| 1);
        self.cursor.row = @min(self.cursor.row, self.rows -| 1);
    }

    pub fn set_cursor(self: *Grid, col: u16, row: u16) void {
        self.cursor.col = @min(col, if (self.cols > 0) self.cols - 1 else 0);
        self.cursor.row = @min(row, if (self.rows > 0) self.rows - 1 else 0);
    }

    // --- Alt screen ---

    pub fn enter_alt_screen(self: *Grid) void {
        if (self.is_alt_screen) return;
        const total = @as(usize, self.cols) * @as(usize, self.rows);

        // Save main screen.
        const saved = self.allocator.alloc(Cell, total) catch return;
        @memcpy(saved, self.cells);

        self.alt_cells = saved;
        self.alt_cursor = self.cursor;
        self.is_alt_screen = true;

        // Clear screen for alt buffer.
        @memset(self.cells, Cell{});
        self.cursor = .{};
    }

    pub fn leave_alt_screen(self: *Grid) void {
        if (!self.is_alt_screen) return;

        if (self.alt_cells) |saved| {
            const total = @as(usize, self.cols) * @as(usize, self.rows);
            const copy_len = @min(saved.len, total);
            @memcpy(self.cells[0..copy_len], saved[0..copy_len]);
            self.allocator.free(saved);
            self.alt_cells = null;
        }

        self.cursor = self.alt_cursor;
        self.is_alt_screen = false;
    }

    // --- Viewport scrollback ---

    pub fn scroll_viewport(self: *Grid, delta: i32) void {
        const max: i32 = @intCast(self.scrollback.items.len);
        const current: i32 = @intCast(self.viewport_offset);
        const new_offset = @max(0, @min(current + delta, max));
        self.viewport_offset = @intCast(new_offset);
    }

    // --- Resize ---

    pub fn resize(self: *Grid, new_cols: u16, new_rows: u16) void {
        const new_total = @as(usize, new_cols) * @as(usize, new_rows);
        const new_cells = self.allocator.alloc(Cell, new_total) catch return;
        @memset(new_cells, Cell{});

        const copy_rows = @min(self.rows, new_rows);
        const copy_cols = @min(self.cols, new_cols);
        var row: u16 = 0;
        while (row < copy_rows) : (row += 1) {
            var col: u16 = 0;
            while (col < copy_cols) : (col += 1) {
                const old_idx = @as(usize, row) * @as(usize, self.cols) + @as(usize, col);
                const new_idx = @as(usize, row) * @as(usize, new_cols) + @as(usize, col);
                new_cells[new_idx] = self.cells[old_idx];
            }
        }

        self.allocator.free(self.cells);
        self.cells = new_cells;
        self.cols = new_cols;
        self.rows = new_rows;
        self.cursor.col = @min(self.cursor.col, if (new_cols > 0) new_cols - 1 else 0);
        self.cursor.row = @min(self.cursor.row, if (new_rows > 0) new_rows - 1 else 0);
        self.scroll_top = 0;
        self.scroll_bottom = new_rows;
    }
};
