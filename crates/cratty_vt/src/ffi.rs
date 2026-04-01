//! Raw FFI declarations for the Zig VT library.

use std::ffi::c_void;

pub type ZigGrid = c_void;
pub type ZigParser = c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ZigCell {
    pub char: u32,
    pub fg_r: u8,
    pub fg_g: u8,
    pub fg_b: u8,
    pub bg_r: u8,
    pub bg_g: u8,
    pub bg_b: u8,
    pub attrs: u8,
    pub _pad: u8,
}

unsafe extern "C" {
    pub fn cratty_grid_new(cols: u16, rows: u16, scrollback_limit: u32) -> *mut ZigGrid;
    pub fn cratty_grid_destroy(grid: *mut ZigGrid);
    pub fn cratty_parser_new() -> *mut ZigParser;
    pub fn cratty_parser_destroy(parser: *mut ZigParser);
    pub fn cratty_parser_feed(
        parser: *mut ZigParser,
        grid: *mut ZigGrid,
        data: *const u8,
        len: usize,
    ) -> usize;
    pub fn cratty_grid_resize(grid: *mut ZigGrid, cols: u16, rows: u16);
    pub fn cratty_grid_get_cell(grid: *const ZigGrid, col: u16, row: u16) -> *const ZigCell;
    pub fn cratty_grid_cursor_col(grid: *const ZigGrid) -> u16;
    pub fn cratty_grid_cursor_row(grid: *const ZigGrid) -> u16;
    pub fn cratty_grid_cols(grid: *const ZigGrid) -> u16;
    pub fn cratty_grid_rows(grid: *const ZigGrid) -> u16;
    pub fn cratty_grid_cursor_visible(grid: *const ZigGrid) -> bool;
    pub fn cratty_grid_scroll_viewport(grid: *mut ZigGrid, delta: i32);
}
