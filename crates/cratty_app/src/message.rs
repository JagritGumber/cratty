use iced::Color;

use crate::style::{TAB_CHAR_W, TAB_ICON_GAP, TAB_ICON_W, TAB_PAD_H};

#[derive(Debug, Clone)]
pub enum Message {
    TermEvent(iced_term::Event),
    NewTab,
    DuplicateTab(usize),
    CloseTab(usize),
    SwitchTab(usize),
    DragWindow,
    Minimize,
    Maximize,
    CloseWindow,
    ToggleTabMenu(u64),
    CloseTabMenu,
    StartRename(usize),
    RenameInput(String),
    ConfirmRename,
    SetTabColor(u64, Option<Color>),
    OpenColorSubmenu,
    EscapePressed,
}

pub struct Tab {
    pub id: u64,
    pub title: String,
    pub custom_title: Option<String>,
    pub color: Option<Color>,
    pub term: iced_term::Terminal,
}

impl Tab {
    pub fn display_title(&self) -> &str {
        self.custom_title.as_deref().unwrap_or(&self.title)
    }

    pub fn estimated_width(&self, is_renaming: bool) -> f32 {
        let text_w = if is_renaming {
            120.0
        } else {
            self.display_title().chars().count().min(20) as f32 * TAB_CHAR_W
        };
        text_w + TAB_ICON_GAP + TAB_ICON_W + TAB_ICON_GAP + TAB_ICON_W + TAB_PAD_H * 2.0
    }
}

pub struct RenameState {
    pub tab_id: u64,
    pub input: String,
}
