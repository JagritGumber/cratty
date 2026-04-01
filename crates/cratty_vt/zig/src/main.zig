const std = @import("std");
const cell = @import("cell.zig");
const parser = @import("parser.zig");
const grid = @import("grid.zig");

pub const Cell = cell.Cell;
pub const Grid = grid.Grid;
pub const Parser = parser.Parser;

// ============================================================================
// C ABI exports for Rust FFI
// ============================================================================

export fn cratty_grid_new(cols: u16, rows: u16, scrollback_limit: u32) ?*Grid {
    const g = grid.Grid.init(
        std.heap.c_allocator,
        cols,
        rows,
        scrollback_limit,
    ) catch return null;

    const ptr = std.heap.c_allocator.create(Grid) catch return null;
    ptr.* = g;
    return ptr;
}

export fn cratty_grid_destroy(g: ?*Grid) void {
    if (g) |ptr| {
        ptr.deinit();
        std.heap.c_allocator.destroy(ptr);
    }
}

export fn cratty_parser_new() ?*Parser {
    const p = std.heap.c_allocator.create(Parser) catch return null;
    p.* = Parser.init();
    return p;
}

export fn cratty_parser_destroy(p: ?*Parser) void {
    if (p) |ptr| {
        std.heap.c_allocator.destroy(ptr);
    }
}

export fn cratty_parser_feed(p: ?*Parser, g: ?*Grid, data: [*]const u8, len: usize) usize {
    const par = p orelse return 0;
    const grd = g orelse return 0;
    par.feed(grd, data[0..len]);
    return len;
}

export fn cratty_grid_resize(g: ?*Grid, cols: u16, rows: u16) void {
    if (g) |ptr| ptr.resize(cols, rows);
}

export fn cratty_grid_get_cell(g: ?*Grid, col: u16, row: u16) ?*const Cell {
    const grd = g orelse return null;
    return grd.get_cell(col, row);
}

export fn cratty_grid_cursor_col(g: ?*Grid) u16 {
    return if (g) |grd| grd.cursor.col else 0;
}

export fn cratty_grid_cursor_row(g: ?*Grid) u16 {
    return if (g) |grd| grd.cursor.row else 0;
}

export fn cratty_grid_cols(g: ?*Grid) u16 {
    return if (g) |grd| grd.cols else 0;
}

export fn cratty_grid_rows(g: ?*Grid) u16 {
    return if (g) |grd| grd.rows else 0;
}

export fn cratty_grid_cursor_visible(g: ?*Grid) bool {
    return if (g) |grd| grd.cursor_visible else true;
}

export fn cratty_grid_scroll_viewport(g: ?*Grid, delta: i32) void {
    if (g) |ptr| ptr.scroll_viewport(delta);
}
