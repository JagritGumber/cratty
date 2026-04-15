use iced::keyboard;
use iced::{event, Subscription};

use crate::message::Message;

pub fn input_subscription() -> Subscription<Message> {
    let key_sub = event::listen_with(|evt, _status, _window| {
        if let iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key, modifiers, ..
        }) = evt {
            if key == keyboard::Key::Named(keyboard::key::Named::Escape) {
                return Some(Message::EscapePressed);
            }
            if modifiers.shift() && !modifiers.alt() && !modifiers.control() {
                if let keyboard::Key::Named(named) = &key {
                    match named {
                        keyboard::key::Named::PageUp =>
                            return Some(Message::ScrollTermUp),
                        keyboard::key::Named::PageDown =>
                            return Some(Message::ScrollTermDown),
                        _ => {}
                    }
                }
            }
            if modifiers.control() && modifiers.shift() && !modifiers.alt() {
                if let keyboard::Key::Character(c) = &key {
                    match c.as_str() {
                        "C" | "c" => return Some(Message::CopyTerminal),
                        "V" | "v" => return Some(Message::PasteTerminal),
                        _ => {}
                    }
                }
            }
            if modifiers.alt() && modifiers.shift() && !modifiers.control() {
                if let keyboard::Key::Named(named) = &key {
                    match named {
                        keyboard::key::Named::ArrowLeft =>
                            return Some(Message::MovePaneLeft),
                        keyboard::key::Named::ArrowRight =>
                            return Some(Message::MovePaneRight),
                        _ => {}
                    }
                }
            }
            if modifiers.control() && !modifiers.shift() && !modifiers.alt() {
                if let keyboard::Key::Character(c) = &key {
                    if matches!(c.as_str(), "S" | "s") {
                        return Some(Message::SaveFocusedFile);
                    }
                }
            }
            if !modifiers.control() && !modifiers.alt() && !modifiers.shift() {
                if let keyboard::Key::Named(named) = &key {
                    match named {
                        keyboard::key::Named::ArrowUp => return Some(Message::FilePickerMove(-1)),
                        keyboard::key::Named::ArrowDown => return Some(Message::FilePickerMove(1)),
                        _ => {}
                    }
                }
            }
            if modifiers.alt() && !modifiers.control() && !modifiers.shift() {
                if let keyboard::Key::Character(c) = &key {
                    match c.as_str().to_ascii_lowercase().as_str() {
                        "t" => return Some(Message::NewWorkspace),
                        "w" => return Some(Message::CloseFocusedPane),
                        "n" => return Some(Message::NewPane),
                        "b" => return Some(Message::ToggleSidebar),
                        "r" => return Some(Message::CyclePresetWidth),
                        "f" => return Some(Message::ToggleMaximizePane),
                        "e" => return Some(Message::OpenFilePicker),
                        _ => {}
                    }
                }
                if let keyboard::Key::Character(c) = &key {
                    if c.as_str() == "-" { return Some(Message::ShrinkPane); }
                    if c.as_str() == "=" { return Some(Message::GrowPane); }
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
    let resize_sub = iced::window::resize_events()
        .map(|(_id, size)| Message::WindowResized(size));
    Subscription::batch([key_sub, resize_sub])
}

pub fn tick_subscription() -> Subscription<Message> {
    iced::time::every(std::time::Duration::from_millis(16))
        .map(|_| Message::Tick)
}
