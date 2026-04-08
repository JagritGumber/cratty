use cratty_core::ColumnWidth;

#[test]
fn proportion_resolve() {
    let w = ColumnWidth::Proportion(0.5);
    assert!((w.resolve(1000.0) - 500.0).abs() < 0.01);
}

#[test]
fn fixed_resolve_ignores_viewport() {
    let w = ColumnWidth::Fixed(400.0);
    assert!((w.resolve(1000.0) - 400.0).abs() < 0.01);
    assert!((w.resolve(2000.0) - 400.0).abs() < 0.01);
}

#[test]
fn adjust_proportion_clamps_low() {
    let w = ColumnWidth::Proportion(0.1);
    let adjusted = w.adjust(-1.0);
    let resolved = adjusted.resolve(1000.0);
    assert!(resolved >= 100.0, "proportion must not go below 0.1");
}

#[test]
fn adjust_proportion_clamps_high() {
    let w = ColumnWidth::Proportion(0.9);
    let adjusted = w.adjust(1.0);
    let resolved = adjusted.resolve(1000.0);
    assert!(resolved <= 1000.0, "proportion must not exceed 1.0");
}

#[test]
fn adjust_fixed_clamps_minimum() {
    let w = ColumnWidth::Fixed(100.0);
    let adjusted = w.adjust(-2.0);
    let resolved = adjusted.resolve(1000.0);
    assert!(resolved >= 100.0, "fixed must not go below 100px");
}

#[test]
fn same_as_proportion_within_tolerance() {
    let a = ColumnWidth::Proportion(0.500);
    let b = ColumnWidth::Proportion(0.505);
    assert!(a.same_as(b));
}

#[test]
fn same_as_proportion_outside_tolerance() {
    let a = ColumnWidth::Proportion(0.5);
    let b = ColumnWidth::Proportion(0.52);
    assert!(!a.same_as(b));
}

#[test]
fn same_as_fixed_within_tolerance() {
    let a = ColumnWidth::Fixed(400.0);
    let b = ColumnWidth::Fixed(400.3);
    assert!(a.same_as(b));
}

#[test]
fn same_as_mismatched_variants() {
    let a = ColumnWidth::Proportion(0.5);
    let b = ColumnWidth::Fixed(500.0);
    assert!(!a.same_as(b));
}

#[test]
fn default_presets_count() {
    let presets = cratty_core::column_width::default_presets();
    assert_eq!(presets.len(), 3);
}
