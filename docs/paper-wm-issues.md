# Paper Window Manager - Known Issues & TODO

Tracking issues discovered during paper WM implementation.

## Open Issues

### Visual

- [ ] Pane border only shows on focused pane (1px blue) - should there be subtle borders on all panes?
- [ ] Sidebar has no resize handle - width is hardcoded at 200px
- [ ] No visual indicator of which direction you can scroll (no peek of adjacent panes)
- [ ] Home screen still references "Ctrl+Shift+T" hint but that creates a workspace, not a pane

### Behavior

- [ ] Ctrl+Shift+Left/Right may conflict with terminal apps that use those keys
- [ ] No way to close individual panes from UI (only via shell exit or ClosePane message)
- [ ] No way to reorder panes within a strip
- [ ] Workspace context menu (rename, color, delete) removed during sidebar migration - needs new UI
- [ ] Tab menu and color submenu code (menu.rs) now unused - clean up or repurpose for sidebar

### Paper WM Core

- [ ] **BLOCKER: iced_term doesn't render inside horizontal scrollable** -- TWO separate issues found:
  1. iced_term passes Size::ZERO as intrinsic size in layout() (fix ready in local fork, PR pending to Harzu/iced_term)
  2. Even with correct layout sizes, TerminalView draw() produces nothing inside scrollable -- likely a rendering/clipping issue with scrollable's viewport translation. Layout debug shows correct dimensions but terminals stay blank.
  - iced's scrollable behavior (compression for Fill children) is CORRECT by design, not a bug (closed iced-rs/iced#3299)
  - Current workaround: FillPortion layout showing focused + adjacent panes (no scrollable)
- [ ] All panes are full viewport width - no configurable widths yet
- [ ] No smooth scroll animation
- [ ] No side-by-side pane view (blocked by scrollable issue above)
- [ ] Scroll position not preserved when switching workspaces

### Architecture

- [ ] main.rs still ~400 lines (update function is the bulk)
- [ ] Unused imports/code warnings from removed tab menu system

## Completed

- [x] Basic sidebar with workspace list
- [x] Titlebar simplified to window chrome only
- [x] Strip view with horizontal scrolling
- [x] Pane dividers between terminals
- [x] Focus indicator (blue border on active pane)
- [x] Ctrl+Shift+N to add pane to workspace
- [x] Ctrl+Shift+Left/Right to navigate between panes
