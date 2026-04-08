use cratty_core::{FocusState, FocusTarget, IdGen, InputMode};

#[test]
fn default_focus_is_none() {
    let state = FocusState::new();
    assert_eq!(state.target, FocusTarget::None);
    assert_eq!(state.mode, InputMode::Normal);
}

#[test]
fn focus_pane_sets_target() {
    let mut state = FocusState::new();
    let mut ids = IdGen::new();
    let pane = ids.next_pane();

    state.focus_pane(pane);
    assert_eq!(state.target, FocusTarget::Pane(pane));
    assert_eq!(state.focused_pane(), Some(pane));
}

#[test]
fn focus_pane_resets_to_normal_mode() {
    let mut state = FocusState::new();
    let mut ids = IdGen::new();
    let pane = ids.next_pane();

    state.enter_navigate();
    assert!(state.is_navigating());

    state.focus_pane(pane);
    assert_eq!(state.mode, InputMode::Normal);
}

#[test]
fn focus_sidebar_sets_target() {
    let mut state = FocusState::new();
    state.focus_sidebar();
    assert_eq!(state.target, FocusTarget::Sidebar);
    assert_eq!(state.focused_pane(), None);
}

#[test]
fn navigate_mode_toggle() {
    let mut state = FocusState::new();
    assert!(!state.is_navigating());

    state.enter_navigate();
    assert!(state.is_navigating());
    assert_eq!(state.mode, InputMode::Navigate);

    state.exit_navigate();
    assert!(!state.is_navigating());
    assert_eq!(state.mode, InputMode::Normal);
}

#[test]
fn focused_pane_returns_none_for_sidebar() {
    let mut state = FocusState::new();
    state.focus_sidebar();
    assert_eq!(state.focused_pane(), None);
}

#[test]
fn focused_pane_returns_none_for_no_focus() {
    let state = FocusState::new();
    assert_eq!(state.focused_pane(), None);
}

#[test]
fn default_trait_matches_new() {
    let a = FocusState::new();
    let b = FocusState::default();
    assert_eq!(a.target, b.target);
    assert_eq!(a.mode, b.mode);
}

#[test]
fn input_mode_default_is_normal() {
    let mode = InputMode::default();
    assert_eq!(mode, InputMode::Normal);
}

#[test]
fn switching_pane_focus() {
    let mut state = FocusState::new();
    let mut ids = IdGen::new();
    let p0 = ids.next_pane();
    let p1 = ids.next_pane();

    state.focus_pane(p0);
    assert_eq!(state.focused_pane(), Some(p0));

    state.focus_pane(p1);
    assert_eq!(state.focused_pane(), Some(p1));
}
