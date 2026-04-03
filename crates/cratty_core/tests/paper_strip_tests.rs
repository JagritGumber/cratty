use cratty_core::{IdGen, PaperStrip, ViewOffset};

#[test]
fn push_and_focus() {
    let mut ids = IdGen::new();
    let mut strip = PaperStrip::new();
    let p0 = ids.next_pane();
    let p1 = ids.next_pane();

    strip.push(p0);
    assert_eq!(strip.focused_pane(), Some(p0));

    strip.push(p1);
    assert_eq!(strip.focused_pane(), Some(p1));
    assert_eq!(strip.focus_idx, 1);
}

#[test]
fn focus_navigation() {
    let mut ids = IdGen::new();
    let mut strip = PaperStrip::new();
    let p0 = ids.next_pane();
    let p1 = ids.next_pane();
    let p2 = ids.next_pane();
    strip.push(p0);
    strip.push(p1);
    strip.push(p2);

    assert!(!strip.focus_right()); // already at end
    assert!(strip.focus_left());
    assert_eq!(strip.focused_pane(), Some(p1));
    assert!(strip.focus_left());
    assert_eq!(strip.focused_pane(), Some(p0));
    assert!(!strip.focus_left()); // already at start
}

#[test]
fn remove_adjusts_focus() {
    let mut ids = IdGen::new();
    let mut strip = PaperStrip::new();
    let p0 = ids.next_pane();
    let p1 = ids.next_pane();
    let p2 = ids.next_pane();
    strip.push(p0);
    strip.push(p1);
    strip.push(p2);
    // focus is at p2 (idx 2)

    strip.remove(p2);
    assert_eq!(strip.focus_idx, 1);
    assert_eq!(strip.focused_pane(), Some(p1));

    strip.remove(p0);
    assert_eq!(strip.focus_idx, 0);
    assert_eq!(strip.focused_pane(), Some(p1));
}

#[test]
fn remove_nonexistent_returns_false() {
    let mut ids = IdGen::new();
    let mut strip = PaperStrip::new();
    let p0 = ids.next_pane();
    let phantom = ids.next_pane();
    strip.push(p0);
    assert!(!strip.remove(phantom));
}

#[test]
fn view_offset_animation() {
    let mut offset = ViewOffset::default();
    assert_eq!(offset.current(), 0.0);

    offset.animate_to(100.0);
    assert!(matches!(offset, ViewOffset::Animating { .. }));

    // Tick partway
    assert!(offset.tick(0.5));
    let mid = offset.current();
    assert!(mid > 0.0 && mid < 100.0);

    // Tick to completion
    assert!(!offset.tick(1.0));
    assert_eq!(offset.current(), 100.0);
}

#[test]
fn view_offset_snap_when_close() {
    let mut offset = ViewOffset::Static(99.8);
    offset.animate_to(100.0);
    // Should snap directly since distance < 0.5
    assert!(matches!(offset, ViewOffset::Static(x) if (x - 100.0).abs() < 0.01));
}
