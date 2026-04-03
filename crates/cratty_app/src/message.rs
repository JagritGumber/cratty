use cratty_core::{PaneId, WorkspaceId};
use iced::Color;

#[derive(Debug, Clone)]
pub enum Message {
    TermEvent(iced_term::Event),
    NewWorkspace,
    DuplicateWorkspace(usize),
    CloseWorkspace(usize),
    SwitchWorkspace(usize),
    DragWindow,
    Minimize,
    Maximize,
    CloseWindow,
    ToggleTabMenu(WorkspaceId),
    CloseTabMenu,
    StartRename(usize),
    RenameInput(String),
    ConfirmRename,
    SetWorkspaceColor(WorkspaceId, Option<Color>),
    OpenColorSubmenu,
    EscapePressed,
}

pub struct RenameState {
    pub workspace_id: WorkspaceId,
    pub input: String,
}
