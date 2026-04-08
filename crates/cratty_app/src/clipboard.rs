use arboard::Clipboard;

/// Write text to the system clipboard.
/// Returns `Ok(())` on success, or an error string for logging.
pub fn set_text(text: &str) -> Result<(), String> {
    Clipboard::new()
        .and_then(|mut cb| cb.set_text(text.to_owned()))
        .map_err(|e| format!("clipboard write failed: {e}"))
}

/// Read text from the system clipboard.
/// Returns the clipboard text or an error string for logging.
pub fn get_text() -> Result<String, String> {
    Clipboard::new()
        .and_then(|mut cb| cb.get_text())
        .map_err(|e| format!("clipboard read failed: {e}"))
}
