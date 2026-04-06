use iced::keyboard;
use iced::{event, Subscription};

use crate::message::Message;

pub fn subscription() -> Subscription<Message> {
    let key_sub = event::listen_with(|evt, _status, _window| {
        if let iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key, modifiers, ..
        }) = evt {
            if key == keyboard::Key::Named(keyboard::key::Named::Escape) {
                return Some(Message::EscapePressed);
            }
            if modifiers.control() && modifiers.shift() {
                if let keyboard::Key::Character(c) = &key {
                    if c.as_str().eq_ignore_ascii_case("t") {
                        return Some(Message::NewWorkspace);
                    }
                    if c.as_str().eq_ignore_ascii_case("n") {
                        return Some(Message::NewPane);
                    }
                    if c.as_str().eq_ignore_ascii_case("b") {
                        return Some(Message::ToggleSidebar);
                    }
                }
                if let keyboard::Key::Named(named) = &key {
                    match named {
                        keyboard::key::Named::ArrowLeft =>
                            return Some(Message::FocusPaneLeft),
                        keyboard::key::Named::ArrowRight =>
                            return Some(Message::FocusPaneRight),
                        _ => {}
                    }
                }
            }
        }
        None
    });
    let tick_sub = iced::time::every(std::time::Duration::from_millis(16))
        .map(|_| Message::Tick);
    Subscription::batch([key_sub, tick_sub])
}
