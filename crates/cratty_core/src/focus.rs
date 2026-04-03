use crate::id::PaneId;

/// What currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusTarget {
    Pane(PaneId),
    Sidebar,
    None,
}

/// Input mode state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Keys go to the focused terminal.
    Normal,
    /// Keys interpreted as navigation commands (like tmux prefix mode).
    Navigate,
    /// Renaming a workspace or pane.
    Rename,
}

impl Default for InputMode {
    fn default() -> Self {
        Self::Normal
    }
}

/// Combined focus and input mode state.
pub struct FocusState {
    pub target: FocusTarget,
    pub mode: InputMode,
}

impl FocusState {
    pub fn new() -> Self {
        Self {
            target: FocusTarget::None,
            mode: InputMode::Normal,
        }
    }

    pub fn focus_pane(&mut self, pane: PaneId) {
        self.target = FocusTarget::Pane(pane);
        self.mode = InputMode::Normal;
    }

    pub fn focus_sidebar(&mut self) {
        self.target = FocusTarget::Sidebar;
        self.mode = InputMode::Normal;
    }

    pub fn enter_navigate(&mut self) {
        self.mode = InputMode::Navigate;
    }

    pub fn exit_navigate(&mut self) {
        self.mode = InputMode::Normal;
    }

    pub fn is_navigating(&self) -> bool {
        self.mode == InputMode::Navigate
    }

    pub fn focused_pane(&self) -> Option<PaneId> {
        match self.target {
            FocusTarget::Pane(id) => Some(id),
            _ => None,
        }
    }
}

impl Default for FocusState {
    fn default() -> Self {
        Self::new()
    }
}
