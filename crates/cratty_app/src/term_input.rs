use iced::keyboard::{self, key::Named, Modifiers};

/// Convert an iced key event into bytes to send to the PTY.
pub fn key_to_bytes(
    key: &keyboard::Key, modifiers: &Modifiers, text: &Option<impl AsRef<str>>,
) -> Option<Vec<u8>> {
    // Ctrl+letter produces control characters (0x01-0x1A)
    if modifiers.control() {
        if let keyboard::Key::Character(c) = key {
            let ch = c.chars().next()?;
            if ch.is_ascii_alphabetic() {
                let ctrl_byte = (ch.to_ascii_lowercase() as u8) - b'a' + 1;
                return Some(vec![ctrl_byte]);
            }
        }
    }

    // Named keys -> ANSI escape sequences
    if let keyboard::Key::Named(named) = key {
        return named_key_bytes(named, modifiers);
    }

    // Regular text input
    if let Some(txt) = text {
        let bytes = txt.as_ref().as_bytes();
        if !bytes.is_empty() {
            return Some(bytes.to_vec());
        }
    }

    None
}

fn named_key_bytes(named: &Named, mods: &Modifiers) -> Option<Vec<u8>> {
    let seq = match named {
        Named::ArrowUp => "\x1b[A",
        Named::ArrowDown => "\x1b[B",
        Named::ArrowRight => "\x1b[C",
        Named::ArrowLeft => "\x1b[D",
        Named::Home => "\x1b[H",
        Named::End => "\x1b[F",
        Named::Insert => "\x1b[2~",
        Named::Delete => "\x1b[3~",
        Named::PageUp => "\x1b[5~",
        Named::PageDown => "\x1b[6~",
        Named::Enter => "\r",
        Named::Backspace => "\x7f",
        Named::Tab if mods.shift() => "\x1b[Z",
        Named::Tab => "\t",
        Named::Escape => "\x1b",
        Named::F1 => "\x1bOP",
        Named::F2 => "\x1bOQ",
        Named::F3 => "\x1bOR",
        Named::F4 => "\x1bOS",
        Named::F5 => "\x1b[15~",
        Named::F6 => "\x1b[17~",
        Named::F7 => "\x1b[18~",
        Named::F8 => "\x1b[19~",
        Named::F9 => "\x1b[20~",
        Named::F10 => "\x1b[21~",
        Named::F11 => "\x1b[23~",
        Named::F12 => "\x1b[24~",
        _ => return None,
    };
    Some(seq.as_bytes().to_vec())
}
