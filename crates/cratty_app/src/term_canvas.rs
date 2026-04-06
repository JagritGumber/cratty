use std::sync::{Arc, Mutex};

use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::Term;
use iced::widget::canvas::{self, Frame, Geometry};
use iced::{Color, Point, Rectangle, Size};

use crate::style::{CELL_H, CELL_W, TERM_FONT_SIZE};
use crate::term_backend::EventProxy;
use crate::term_colors::{ansi_to_iced, BG};

/// Draw the terminal grid onto a Canvas frame.
pub fn draw_grid(
    term: &Arc<FairMutex<Term<EventProxy>>>,
    renderer: &iced::Renderer,
    bounds: Rectangle,
    pending_resize: &Arc<Mutex<Option<(u16, u16)>>>,
) -> Vec<Geometry> {
    let t = term.lock();
    let grid = t.grid();

    // Signal resize if bounds changed
    let new_cols = (bounds.width / CELL_W).floor().max(1.0) as u16;
    let new_rows = (bounds.height / CELL_H).floor().max(1.0) as u16;
    *pending_resize.lock().unwrap() = Some((new_cols, new_rows));

    let mut frame = Frame::new(renderer, bounds.size());
    frame.fill_rectangle(Point::ORIGIN, bounds.size(), BG);

    for indexed in grid.display_iter() {
        let col = indexed.point.column.0 as f32;
        let line = indexed.point.line.0 as f32;
        let x = col * CELL_W;
        let y = line * CELL_H;
        let px = Point::new(x, y);

        let bg_color = ansi_to_iced(indexed.bg);
        if bg_color != BG {
            frame.fill_rectangle(px, Size::new(CELL_W, CELL_H), bg_color);
        }

        let c = indexed.c;
        if c == ' ' || c == '\0' {
            continue;
        }

        frame.fill_text(canvas::Text {
            content: c.to_string(),
            position: px,
            color: ansi_to_iced(indexed.fg),
            size: TERM_FONT_SIZE.into(),
            font: iced::Font::MONOSPACE,
            ..canvas::Text::default()
        });
    }

    // Cursor (screen-relative, line.0 is always 0..screen_lines)
    let cx = grid.cursor.point.column.0 as f32 * CELL_W;
    let cy = grid.cursor.point.line.0 as f32 * CELL_H;
    frame.fill_rectangle(
        Point::new(cx, cy),
        Size::new(CELL_W, CELL_H),
        Color::from_rgba(0.8, 0.8, 0.8, 0.7),
    );

    vec![frame.into_geometry()]
}
