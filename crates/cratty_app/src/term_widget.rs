use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use alacritty_terminal::event::Notify;
use alacritty_terminal::event_loop::Notifier;
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::Term;
use iced::widget::canvas::{self, Canvas, Event, Geometry};
use iced::{Element, Length, Rectangle};

use cratty_core::FontMetrics;

use crate::message::Message;
use crate::term_backend::{EventProxy, TermBackend};
use crate::{term_canvas, term_input};

struct TermProgram {
    term: Arc<FairMutex<Term<EventProxy>>>,
    notifier: Notifier,
    pending_resize: Arc<Mutex<Option<(u16, u16)>>>,
    focused: bool,
    metrics: FontMetrics,
}

impl canvas::Program<Message> for TermProgram {
    type State = ();

    fn update(
        &self, _state: &mut (), event: &Event,
        _bounds: Rectangle, _cursor: iced::mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if !self.focused { return None; }
        if let Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key, text, modifiers, ..
        }) = event
        {
            if modifiers.alt() && !modifiers.control() && !modifiers.shift() {
                return None;
            }
            if let Some(bytes) = term_input::key_to_bytes(key, modifiers, text) {
                self.notifier.notify(Cow::Owned(bytes));
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
            &self.pending_resize, self.focused, self.metrics,
        )
    }
}

pub fn view(
    backend: &TermBackend, focused: bool, metrics: FontMetrics,
) -> Element<'_, Message> {
    let program = TermProgram {
        term: backend.term.clone(),
        notifier: Notifier(backend.sender.clone()),
        pending_resize: backend.pending_resize.clone(),
        focused,
        metrics,
    };

    Canvas::new(program)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
