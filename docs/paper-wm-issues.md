# Paper Window Manager - Known Issues & TODO

Tracking issues discovered during paper WM implementation.

## Open Issues

### Visual

- [ ] Sidebar has no resize handle - width is hardcoded at 200px
- [ ] No visual indicator of which direction you can scroll (no peek of adjacent panes)
- [ ] Home screen references "Ctrl+Shift+T" but that creates a workspace, not a pane
- [ ] Unfocused panes show dim overlay but terminal cursor stays filled block (needs outline cursor)

### Behavior

- [ ] Ctrl+Shift+N leaks ^N character into the terminal (keyboard event not consumed)
- [ ] Ctrl+Shift+Left/Right may conflict with terminal apps that use those keys
- [ ] No way to close individual panes from UI (only via shell exit)
- [ ] No way to reorder panes within a strip
- [ ] Workspace context menu (rename, color, delete) not yet added to sidebar

### Paper WM Core

- [ ] **True paper WM clipping not implemented** -- FillPortion divides space equally (tiling behavior). True paper WM needs each pane to keep its own width with viewport clipping. Requires scrollable support.
- [ ] **Scrollable rendering with Canvas** -- Canvas uses local coords + with_translation (correct pattern). Not yet tested inside scrollable. See docs/iced-term-scrollable-investigation.md.
- [ ] All panes are equal width - no configurable widths yet
- [ ] No smooth scroll animation between panes
- [ ] Scroll position not preserved when switching workspaces
- [ ] Terminal resize feedback loop: never use terminal layout size for viewport width calculation (see iced-patterns.md)

### Architecture

- [ ] main.rs still ~290 lines (update function is the bulk)
- [ ] term_canvas.rs uses hardcoded cell dimensions (8.2x18.2) - should measure from font

## Completed

- [x] Basic sidebar with workspace list
- [x] Titlebar simplified to window chrome only
- [x] Pane dividers between terminals
- [x] Focus indicator (blue border + dim overlay on unfocused)
- [x] Ctrl+Shift+N to add pane to workspace
- [x] Ctrl+Shift+Left/Right to navigate between panes
- [x] Replaced iced_term with own Canvas-based renderer (PR #10)
- [x] term_backend.rs wrapping alacritty_terminal directly
- [x] term_canvas.rs drawing grid cells on iced Canvas
- [x] term_widget.rs Canvas Program with keyboard input
- [x] Dropped iced_term dependency
- [x] iced scrollable compression investigated (correct by design, closed iced-rs/iced#3299)
- [x] iced_term coordinate bugs identified (Size::ZERO intrinsic + absolute coords in local frame)
- [x] Viewport width feedback loop bug found and documented
