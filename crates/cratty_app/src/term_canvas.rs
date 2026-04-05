use std::sync::Arc;

use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::Term;
use alacritty_terminal::vte::ansi::{self, NamedColor};
use iced::widget::canvas::{self, Frame, Geometry};
use iced::{Color, Point, Rectangle, Size};

use crate::term_backend::EventProxy;

/// Default ANSI color palette (matches most terminal defaults).
const ANSI_COLORS: [Color; 16] = [
    Color::from_rgb(0.0, 0.0, 0.0),           // Black
    Color::from_rgb(0.8, 0.0, 0.0),           // Red
    Color::from_rgb(0.0, 0.8, 0.0),           // Green
    Color::from_rgb(0.8, 0.8, 0.0),           // Yellow
    Color::from_rgb(0.0, 0.0, 0.8),           // Blue
    Color::from_rgb(0.8, 0.0, 0.8),           // Magenta
    Color::from_rgb(0.0, 0.8, 0.8),           // Cyan
    Color::from_rgb(0.75, 0.75, 0.75),        // White
    Color::from_rgb(0.5, 0.5, 0.5),           // Bright Black
    Color::from_rgb(1.0, 0.0, 0.0),           // Bright Red
    Color::from_rgb(0.0, 1.0, 0.0),           // Bright Green
    Color::from_rgb(1.0, 1.0, 0.0),           // Bright Yellow
    Color::from_rgb(0.3, 0.5, 1.0),           // Bright Blue
    Color::from_rgb(1.0, 0.0, 1.0),           // Bright Magenta
    Color::from_rgb(0.0, 1.0, 1.0),           // Bright Cyan
    Color::WHITE,                              // Bright White
];

const BG: Color = Color::from_rgb(0.094, 0.094, 0.094);
const FG: Color = Color::from_rgb(0.8, 0.8, 0.8);

pub fn ansi_to_iced(color: ansi::Color) -> Color {
    match color {
        ansi::Color::Named(n) => match n {
            NamedColor::Background => BG,
            NamedColor::Foreground | NamedColor::Cursor => FG,
            _ => {
                let idx = n as usize;
                if idx < 16 { ANSI_COLORS[idx] } else { FG }
            }
        },
        ansi::Color::Indexed(i) => {
            if (i as usize) < 16 { ANSI_COLORS[i as usize] } else { FG }
        }
        ansi::Color::Spec(rgb) => {
            Color::from_rgb8(rgb.r, rgb.g, rgb.b)
        }
    }
}

/// Draw the terminal grid onto a Canvas frame.
pub fn draw_grid(
    term: &Arc<FairMutex<Term<EventProxy>>>,
    renderer: &iced::Renderer,
    bounds: Rectangle,
    cell_w: f32,
    cell_h: f32,
    font_size: f32,
) -> Vec<Geometry> {
    let t = term.lock();
    let cols = t.columns();
    let lines = t.screen_lines();
    let grid = t.grid();

    let mut frame = Frame::new(renderer, bounds.size());

    // Background fill
    frame.fill_rectangle(Point::ORIGIN, bounds.size(), BG);

    for indexed in grid.display_iter() {
        let col = indexed.point.column.0 as f32;
        let line = indexed.point.line.0 as f32;
        let x = col * cell_w;
        let y = line * cell_h;

        let bg_color = ansi_to_iced(indexed.bg);
        if bg_color != BG {
            frame.fill_rectangle(
                Point::new(x, y),
                Size::new(cell_w, cell_h),
                bg_color,
            );
        }

        let c = indexed.c;
        if c == ' ' || c == '\0' {
            continue;
        }

        let fg_color = ansi_to_iced(indexed.fg);
        frame.fill_text(canvas::Text {
            content: c.to_string(),
            position: Point::new(x, y),
            color: fg_color,
            size: font_size.into(),
            font: iced::Font::MONOSPACE,
            ..canvas::Text::default()
        });
    }

    // Cursor
    let cursor_point = grid.cursor.point;
    let cx = cursor_point.column.0 as f32 * cell_w;
    let cy = cursor_point.line.0 as f32 * cell_h;
    frame.fill_rectangle(
        Point::new(cx, cy),
        Size::new(cell_w, cell_h),
        Color::from_rgba(0.8, 0.8, 0.8, 0.7),
    );

    vec![frame.into_geometry()]
}
