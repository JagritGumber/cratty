use crate::ffi;

/// Terminal cell attributes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CellAttrs {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub inverse: bool,
    pub dim: bool,
}

impl CellAttrs {
    fn from_raw(raw: u8) -> Self {
        Self {
            bold: raw & 0x01 != 0,
            italic: raw & 0x02 != 0,
            underline: raw & 0x04 != 0,
            strikethrough: raw & 0x08 != 0,
            inverse: raw & 0x10 != 0,
            dim: raw & 0x20 != 0,
        }
    }
}

/// A terminal cell.
#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub ch: char,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
    pub attrs: CellAttrs,
}

impl Cell {
    fn from_zig(raw: &ffi::ZigCell) -> Self {
        Self {
            ch: char::from_u32(raw.char).unwrap_or(' '),
            fg: [raw.fg_r, raw.fg_g, raw.fg_b],
            bg: [raw.bg_r, raw.bg_g, raw.bg_b],
            attrs: CellAttrs::from_raw(raw.attrs),
        }
    }
}

/// Cursor position.
#[derive(Debug, Clone, Copy, Default)]
pub struct Cursor {
    pub col: u16,
    pub row: u16,
}

/// A snapshot of the grid state for rendering.
pub struct GridSnapshot {
    pub cols: u16,
    pub rows: u16,
    pub cells: Vec<Cell>,
    pub cursor: Cursor,
    pub cursor_visible: bool,
}

/// Safe wrapper around the Zig VT grid + parser.
pub struct Grid {
    grid_ptr: *mut ffi::ZigGrid,
    parser_ptr: *mut ffi::ZigParser,
    cols: u16,
    rows: u16,
}

// The Zig grid is not thread-safe, but we ensure single-threaded access
// via the terminal's dedicated thread.
unsafe impl Send for Grid {}

impl Grid {
    /// Create a new terminal grid.
    pub fn new(cols: u16, rows: u16, scrollback_limit: u32) -> Self {
        unsafe {
            let grid_ptr = ffi::cratty_grid_new(cols, rows, scrollback_limit);
            let parser_ptr = ffi::cratty_parser_new();
            assert!(!grid_ptr.is_null(), "Failed to allocate grid");
            assert!(!parser_ptr.is_null(), "Failed to allocate parser");
            Self {
                grid_ptr,
                parser_ptr,
                cols,
                rows,
            }
        }
    }

    /// Feed raw bytes from the PTY into the VT parser.
    pub fn feed(&mut self, data: &[u8]) {
        unsafe {
            ffi::cratty_parser_feed(self.parser_ptr, self.grid_ptr, data.as_ptr(), data.len());
        }
    }

    /// Resize the terminal grid.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        unsafe {
            ffi::cratty_grid_resize(self.grid_ptr, cols, rows);
        }
        self.cols = cols;
        self.rows = rows;
    }

    /// Get the cursor position.
    pub fn cursor(&self) -> Cursor {
        unsafe {
            Cursor {
                col: ffi::cratty_grid_cursor_col(self.grid_ptr),
                row: ffi::cratty_grid_cursor_row(self.grid_ptr),
            }
        }
    }

    /// Get a cell at (col, row).
    pub fn get_cell(&self, col: u16, row: u16) -> Option<Cell> {
        unsafe {
            let ptr = ffi::cratty_grid_get_cell(self.grid_ptr, col, row);
            if ptr.is_null() {
                None
            } else {
                Some(Cell::from_zig(&*ptr))
            }
        }
    }

    /// Take a snapshot of the entire grid for rendering.
    pub fn snapshot(&self) -> GridSnapshot {
        let mut cells = Vec::with_capacity(self.cols as usize * self.rows as usize);
        for row in 0..self.rows {
            for col in 0..self.cols {
                cells.push(self.get_cell(col, row).unwrap_or(Cell {
                    ch: ' ',
                    fg: [204, 204, 204],
                    bg: [30, 30, 30],
                    attrs: CellAttrs::default(),
                }));
            }
        }
        GridSnapshot {
            cols: self.cols,
            rows: self.rows,
            cells,
            cursor: self.cursor(),
            cursor_visible: self.cursor_visible(),
        }
    }

    /// Whether the cursor should be visible.
    pub fn cursor_visible(&self) -> bool {
        unsafe { ffi::cratty_grid_cursor_visible(self.grid_ptr) }
    }

    /// Scroll the viewport into scrollback history.
    pub fn scroll_viewport(&mut self, delta: i32) {
        unsafe {
            ffi::cratty_grid_scroll_viewport(self.grid_ptr, delta);
        }
    }

    pub fn cols(&self) -> u16 {
        self.cols
    }

    pub fn rows(&self) -> u16 {
        self.rows
    }
}

impl Drop for Grid {
    fn drop(&mut self) {
        unsafe {
            ffi::cratty_parser_destroy(self.parser_ptr);
            ffi::cratty_grid_destroy(self.grid_ptr);
        }
    }
}
