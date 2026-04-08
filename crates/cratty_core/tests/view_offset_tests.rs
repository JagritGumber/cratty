use cratty_core::ViewOffset;

#[test]
fn default_is_static_zero() {
    let offset = ViewOffset::default();
    assert_eq!(offset.current(), 0.0);
}

#[test]
fn static_current_returns_value() {
    let offset = ViewOffset::Static(42.0);
    assert!((offset.current() - 42.0).abs() < 0.01);
}

#[test]
fn animate_to_starts_animation() {
    let mut offset = ViewOffset::Static(0.0);
    offset.animate_to(100.0);
    assert!(matches!(offset, ViewOffset::Animating { .. }));
}

#[test]
fn animate_to_snaps_when_close() {
    let mut offset = ViewOffset::Static(99.8);
    offset.animate_to(100.0);
    assert!(matches!(offset, ViewOffset::Static(_)));
    assert!((offset.current() - 100.0).abs() < 0.01);
}

#[test]
fn tick_advances_progress() {
    let mut offset = ViewOffset::Static(0.0);
    offset.animate_to(100.0);
    let still_going = offset.tick(0.3);
    assert!(still_going);
    let pos = offset.current();
    assert!(pos > 0.0 && pos < 100.0);
}

#[test]
fn tick_completes_at_one() {
    let mut offset = ViewOffset::Static(0.0);
    offset.animate_to(100.0);
    let done = offset.tick(1.0);
    assert!(!done);
    assert!((offset.current() - 100.0).abs() < 0.01);
}

#[test]
fn tick_on_static_returns_false() {
    let mut offset = ViewOffset::Static(50.0);
    assert!(!offset.tick(0.1));
    assert!((offset.current() - 50.0).abs() < 0.01);
}

#[test]
fn animation_monotonically_approaches_target() {
    let mut offset = ViewOffset::Static(0.0);
    offset.animate_to(200.0);

    let mut prev = 0.0_f32;
    for _ in 0..10 {
        offset.tick(0.1);
        let cur = offset.current();
        assert!(cur >= prev, "animation must not go backwards");
        prev = cur;
    }
}

#[test]
fn sequential_animate_to_uses_current_position() {
    let mut offset = ViewOffset::Static(0.0);
    offset.animate_to(100.0);
    offset.tick(0.5);
    let mid = offset.current();

    offset.animate_to(200.0);
    if let ViewOffset::Animating { from, .. } = offset {
        assert!((from - mid).abs() < 1.0);
    } else {
        panic!("expected Animating variant");
    }
}
