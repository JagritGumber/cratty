use cratty_core::PaneId;

#[derive(Debug, Clone)]
pub enum Message {
    TermEvent(iced_term::Event),
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
}
