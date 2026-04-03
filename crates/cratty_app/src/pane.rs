use cratty_core::PaneId;
use iced::Color;

/// A terminal pane -- wraps iced_term::Terminal with display metadata.
pub struct Pane {
    pub id: PaneId,
    pub term_id: u64,
    pub terminal: iced_term::Terminal,
    pub title: String,
    pub custom_title: Option<String>,
    pub color: Option<Color>,
}

impl Pane {
    pub fn new(id: PaneId, term_id: u64, terminal: iced_term::Terminal) -> Self {
        Self {
            id,
            term_id,
            terminal,
            title: "Terminal".into(),
            custom_title: None,
            color: None,
        }
    }

    pub fn display_title(&self) -> &str {
        self.custom_title.as_deref().unwrap_or(&self.title)
    }
}
