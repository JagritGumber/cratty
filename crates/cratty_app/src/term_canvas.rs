use std::sync::{Arc, Mutex};

use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::{cell::Flags, Term};
use iced::widget::canvas::{self, Frame, Geometry};
use iced::{Color, Point, Rectangle, Size};

use cratty_core::FontMetrics;

use crate::term_backend::EventProxy;
use crate::term_colors::{ansi_to_iced, BG};
use crate::{term_cursor, term_decor};

/// Draw the terminal grid onto a Canvas frame.
pub fn draw_grid(
    term: &Arc<FairMutex<Term<EventProxy>>>,
    renderer: &iced::Renderer,
    bounds: Rectangle,
    pending_resize: &Arc<Mutex<Option<(u16, u16)>>>,
    focused: bool,
    metrics: FontMetrics,
) -> Vec<Geometry> {
    let FontMetrics { font_size, cell_w, cell_h } = metrics;
    let t = term.lock();
    let grid = t.grid();

    let new_cols = (bounds.width / cell_w).floor().max(1.0) as u16;
    let new_rows = (bounds.height / cell_h).floor().max(1.0) as u16;
    *pending_resize.lock().unwrap() = Some((new_cols, new_rows));

    let mut frame = Frame::new(renderer, bounds.size());
    frame.fill_rectangle(Point::ORIGIN, bounds.size(), BG);

    for indexed in grid.display_iter() {
        let flags = indexed.flags;
        if flags.contains(Flags::WIDE_CHAR_SPACER) {
            continue;
        }

        let col = indexed.point.column.0 as f32;
        let line = indexed.point.line.0 as f32;
        let px = Point::new(col * cell_w, line * cell_h);
        let is_bold = flags.contains(Flags::BOLD);

        let (fg_raw, bg_raw) = if flags.contains(Flags::INVERSE) {
            (indexed.bg, indexed.fg)
        } else {
            (indexed.fg, indexed.bg)
        };

        let mut fg_color = ansi_to_iced(fg_raw, is_bold);
        let bg_color = ansi_to_iced(bg_raw, false);

        if flags.contains(Flags::DIM) {
            fg_color = Color { a: fg_color.a * 0.5, ..fg_color };
        }
        if flags.contains(Flags::HIDDEN) {
            fg_color = bg_color;
        }

        if bg_color != BG {
            frame.fill_rectangle(px, Size::new(cell_w, cell_h), bg_color);
        }

        let c = indexed.c;
        if c != ' ' && c != '\0' {
            let weight = if is_bold { iced::font::Weight::Bold } else { iced::font::Weight::Normal };
            let style = if flags.contains(Flags::ITALIC) { iced::font::Style::Italic } else { iced::font::Style::Normal };
            let font = iced::Font { weight, style, ..iced::Font::MONOSPACE };
            frame.fill_text(canvas::Text {
                content: c.to_string(),
                position: px,
                color: fg_color,
                size: font_size.into(),
                font,
                ..canvas::Text::default()
            });
        }

        if flags.intersects(Flags::ALL_UNDERLINES) {
            term_decor::draw_underline(&mut frame, px, fg_color, cell_w, cell_h);
        }
        if flags.contains(Flags::STRIKEOUT) {
            term_decor::draw_strikethrough(&mut frame, px, fg_color, cell_w, cell_h);
        }
    }

    let cx = grid.cursor.point.column.0 as f32 * cell_w;
    let cy = grid.cursor.point.line.0 as f32 * cell_h;
    term_cursor::draw_cursor(&mut frame, Point::new(cx, cy), focused, cell_w, cell_h);

    vec![frame.into_geometry()]
}
