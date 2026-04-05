use std::borrow::Cow;
use std::sync::{Arc, Mutex};

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
    pending_resize: Arc<Mutex<Option<(u16, u16)>>>,
}

impl canvas::Program<Message> for TermProgram {
    type State = ();

    fn update(
        &self, _state: &mut (), event: &Event,
        _bounds: Rectangle, _cursor: iced::mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if let Event::Keyboard(iced::keyboard::Event::KeyPressed {
            text, modifiers, ..
        }) = event
        {
            // Don't forward Ctrl+Shift combos -- those are app shortcuts
            if modifiers.control() && modifiers.shift() {
                return None;
            }
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
        term_canvas::draw_grid(&self.term, renderer, bounds, &self.pending_resize)
    }
}

/// Create a Canvas element for a terminal backend.
pub fn view(backend: &TermBackend) -> Element<'_, Message> {
    let program = TermProgram {
        term: backend.term.clone(),
        notifier: Notifier(backend.sender.clone()),
        pending_resize: backend.pending_resize.clone(),
    };

    Canvas::new(program)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
