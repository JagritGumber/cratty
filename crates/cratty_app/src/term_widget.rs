use std::borrow::Cow;
use std::sync::Arc;

use alacritty_terminal::event::Notify;
use alacritty_terminal::event_loop::Notifier;
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::Term;
use iced::widget::canvas::{self, Canvas, Event, Geometry};
use iced::{Element, Length, Rectangle};

use crate::message::Message;
use crate::term_backend::{EventProxy, TermBackend};
use crate::term_canvas;

/// Canvas program that renders a terminal and handles keyboard input.
struct TermProgram {
    term: Arc<FairMutex<Term<EventProxy>>>,
    notifier: Notifier,
    cell_w: f32,
    cell_h: f32,
    font_size: f32,
}

impl canvas::Program<Message> for TermProgram {
    type State = ();

    fn update(
        &self, _state: &mut (), event: &Event,
        _bounds: Rectangle, _cursor: iced::mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if let Event::Keyboard(iced::keyboard::Event::KeyPressed { text, .. }) = event {
            if let Some(txt) = text {
                let bytes = txt.as_bytes();
                if !bytes.is_empty() {
                    self.notifier.notify(Cow::Owned(bytes.to_vec()));
                }
            }
            return Some(canvas::Action::request_redraw());
        }
        None
    }

    fn draw(
        &self, _state: &(), renderer: &iced::Renderer,
        _theme: &iced::Theme, bounds: Rectangle, _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        term_canvas::draw_grid(
            &self.term, renderer, bounds,
            self.cell_w, self.cell_h, self.font_size,
        )
    }
}

/// Create a Canvas element for a terminal backend.
pub fn view(backend: &TermBackend) -> Element<'_, Message> {
    let program = TermProgram {
        term: backend.term.clone(),
        notifier: Notifier(backend.sender.clone()),
        cell_w: 8.2,
        cell_h: 18.2,
        font_size: 14.0,
    };

    Canvas::new(program)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
