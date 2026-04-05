use cratty_core::PaneId;

#[derive(Debug, Clone)]
pub enum Message {
    NewWorkspace,
    CloseWorkspace(usize),
    SwitchWorkspace(usize),
    NewPane,
    ClosePane(PaneId),
    FocusPaneLeft,
    FocusPaneRight,
    DragWindow,
    Minimize,
    Maximize,
    CloseWindow,
    EscapePressed,
    Tick,
}
