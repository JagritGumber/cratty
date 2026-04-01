use crate::session::SessionId;

/// Application-level events.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// A session produced output bytes.
    SessionOutput { id: SessionId, data: Vec<u8> },

    /// A session exited.
    SessionExited { id: SessionId, exit_code: Option<i32> },

    /// User requested a new tab.
    NewTab { profile_index: usize },

    /// User requested closing a tab.
    CloseTab { id: SessionId },

    /// User requested a split.
    Split { direction: SplitDirection },

    /// Terminal was resized.
    Resize { id: SessionId, cols: u16, rows: u16 },

    /// Quake mode toggle.
    ToggleQuake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}
