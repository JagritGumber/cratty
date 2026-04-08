use cratty_core::{IdGen, PaneId, WorkspaceId};

#[test]
fn id_gen_starts_at_zero() {
    let mut ids = IdGen::new();
    let first = ids.next_pane();
    assert_eq!(first, PaneId(0));
}

#[test]
fn id_gen_increments() {
    let mut ids = IdGen::new();
    let a = ids.next_pane();
    let b = ids.next_pane();
    assert_ne!(a, b);
    assert_eq!(b, PaneId(1));
}

#[test]
fn id_gen_shared_counter_across_types() {
    let mut ids = IdGen::new();
    let pane = ids.next_pane();
    let ws = ids.next_workspace();
    assert_eq!(pane, PaneId(0));
    assert_eq!(ws, WorkspaceId(1));
}

#[test]
fn pane_id_display_format() {
    let id = PaneId(42);
    assert_eq!(format!("{id}"), "pane-42");
}

#[test]
fn workspace_id_display_format() {
    let id = WorkspaceId(7);
    assert_eq!(format!("{id}"), "ws-7");
}

#[test]
fn pane_id_equality() {
    assert_eq!(PaneId(5), PaneId(5));
    assert_ne!(PaneId(5), PaneId(6));
}

#[test]
fn workspace_id_equality() {
    assert_eq!(WorkspaceId(3), WorkspaceId(3));
    assert_ne!(WorkspaceId(3), WorkspaceId(4));
}

#[test]
fn id_gen_default_matches_new() {
    let mut a = IdGen::new();
    let mut b = IdGen::default();
    assert_eq!(a.next_pane(), b.next_pane());
}

#[test]
fn pane_id_hash_distinct() {
    use std::collections::HashSet;
    let mut ids = IdGen::new();
    let all: Vec<PaneId> = (0..100).map(|_| ids.next_pane()).collect();
    let set: HashSet<PaneId> = all.iter().copied().collect();
    assert_eq!(set.len(), 100);
}

#[test]
fn pane_id_serialization_round_trip() {
    let id = PaneId(99);
    let json = serde_json::to_string(&id).expect("serialize");
    let restored: PaneId =
        serde_json::from_str(&json).expect("deserialize");
    assert_eq!(id, restored);
}
