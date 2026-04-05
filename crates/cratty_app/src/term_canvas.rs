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
    let cw = u32::from(CELL_W);
    let ch = u32::from(CELL_H);

    // Signal resize if bounds changed
    let new_cols = bounds.width as u16 / CELL_W;
    let new_rows = bounds.height as u16 / CELL_H;
    if new_cols > 0 && new_rows > 0 {
        *pending_resize.lock().unwrap() = Some((new_cols, new_rows));
    }

    let mut frame = Frame::new(renderer, bounds.size());
    frame.fill_rectangle(Point::ORIGIN, bounds.size(), BG);

    for indexed in grid.display_iter() {
        let col = indexed.point.column.0 as u32;
        let line = indexed.point.line.0 as u32;
        let x = col * cw;
        let y = line * ch;
        let px = Point::new(x as f32, y as f32);

        let bg_color = ansi_to_iced(indexed.bg);
        if bg_color != BG {
            frame.fill_rectangle(px, Size::new(cw as f32, ch as f32), bg_color);
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

    // Cursor
    let cursor_point = grid.cursor.point;
    let cx = cursor_point.column.0 as u32 * cw;
    let cy = cursor_point.line.0 as u32 * ch;
    frame.fill_rectangle(
        Point::new(cx as f32, cy as f32),
        Size::new(cw as f32, ch as f32),
        Color::from_rgba(0.8, 0.8, 0.8, 0.7),
    );

    vec![frame.into_geometry()]
}
