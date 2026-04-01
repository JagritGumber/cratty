//! Rust FFI bindings to the Zig VT engine.
//!
//! This crate wraps the Zig-based terminal emulation core (VT parser + grid)
//! and exposes safe Rust types.

mod ffi;
mod grid;

pub use grid::{Cell, CellAttrs, Grid, GridSnapshot, Cursor};
