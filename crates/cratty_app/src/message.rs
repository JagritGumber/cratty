use cratty_core::{PaneId, WorkspaceId};

#[derive(Debug, Clone)]
pub enum Message {
    NewWorkspace,
    CloseWorkspace(usize),
    SwitchWorkspace(usize),
    NewPane,
    ClosePane(PaneId),
    FocusPaneLeft,
    FocusPaneRight,
    CyclePresetWidth,
    ToggleMaximizePane,
    ToggleSidebar,
    RenameWorkspace(WorkspaceId),
    RenameInput(String),
    RenameSubmit,
    SetWorkspaceColor(WorkspaceId, iced::Color),
    ShowWsMenu(usize),
    HideWsMenu,
    WindowResized(iced::Size),
    DragWindow,
    Minimize,
    Maximize,
    CloseWindow,
    EscapePressed,
    Tick,
}
