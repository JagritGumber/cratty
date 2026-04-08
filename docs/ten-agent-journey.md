# The Ten-Agent Sprint: From 30% to Production-Ready

How we took Cratty from a working proof-of-concept to a usable terminal emulator in a single session using 10 parallel AI agents across 4 phases.

## Starting Point

Cratty had the paper WM data model, a custom Canvas-based terminal renderer, and basic keyboard shortcuts. But it was broken for real-world use:

- Colors 16-255 all rendered as the default foreground. `ls --color`, vim, htop looked broken.
- No bold, dim, italic, underline, strikethrough, or inverse text.
- Cursor was a hardcoded gray rectangle. No shape variants.
- Cell dimensions hardcoded to 8.0x18.0 pixels. config.rs existed but was never loaded.
- No clipboard. No mouse support. No scrollback.
- Closing the app lost all workspaces. No session recovery.
- No way to close a pane except typing `exit`.
- Space bar didn't work (Named::Space fell through the match in term_input.rs).
- Home screen showed wrong keybinding ("Ctrl+Shift+T" instead of "Alt+T").
- Only 1 test file. No CI.

## The Space Bar Bug

Before the sprint even started, we found that pressing space did nothing. The root cause: iced reports space as `Key::Named(Named::Space)`. Our `key_to_bytes()` function checked named keys first, and `Named::Space` wasn't in the match. It returned `None` before the text fallback `" "` could fire.

Fix: one line. `Named::Space => " "` in the match arm. A good reminder that "Named" keys in iced include more than just arrows and function keys.

## Phase 1: Make the Terminal Usable

Three agents ran in parallel, each targeting different files.

### Agent 1: Terminal Rendering Fidelity

The biggest visual impact. Created three new files:

**term_palette.rs** -- Full xterm 256-color palette. Indices 0-15 use the existing ANSI colors. 16-231 use the 6x6x6 color cube: `value = if component == 0 { 0 } else { component * 40 + 55 }`. 232-255 use the grayscale ramp: `value = i * 10 + 8`.

**term_decor.rs** -- Underline and strikethrough rendering via `Path::line` + `frame.stroke`. Underline sits 2px above cell bottom, strikethrough at vertical center.

**term_cursor.rs** -- Focused panes get a filled block cursor. Unfocused get a hollow outline. Uses `Path::rectangle` + `frame.stroke` for the outline variant.

The core change was in term_canvas.rs: reading `cell.flags()` from alacritty_terminal's grid iterator. The Flags bitfield gives BOLD, DIM, ITALIC, UNDERLINE, STRIKEOUT, INVERSE, HIDDEN, WIDE_CHAR_SPACER.

Key rendering logic:
```
INVERSE -> swap fg/bg
BOLD -> Font weight Bold + bright color promotion (color 0-7 becomes 8-15)
DIM -> fg alpha * 0.5
ITALIC -> Font style Italic
HIDDEN -> fg = bg
WIDE_CHAR_SPACER -> skip (placeholder for double-width characters)
```

The bold bright-promotion is a terminal convention worth knowing: bold text with a normal ANSI color (0-7) displays as its bright variant (8-15). We pass `is_bold` to the color resolver so it can promote.

### Agent 2: Config Integration + Dynamic Font Metrics

config.rs was 148 lines (over our 100-line limit) and never used. Split into config.rs (structs + defaults, 92 lines) and config_io.rs (load/save/paths, 24 lines).

Created FontMetrics with a simple formula derived from the existing hardcoded values:
```
cell_w = (font_size * 0.6).round()
cell_h = (font_size * 1.28).round()
```

The ratios come from: 8.0/14.0 = 0.571 and 18.0/14.0 = 1.286. Not perfect for every font, but correct for monospace fonts at reasonable sizes.

The hardest part was threading FontMetrics through the rendering pipeline. It touches: term_canvas (draw cells), term_decor (underline position), term_cursor (cursor size), term_widget (carries metrics), strip_view (passes to widgets), app_tick (resize calculations), and app_actions (PTY spawn).

Removing three constants from style.rs and replacing them with a struct parameter that flows through 7 files is tedious but necessary. Hardcoded dimensions break the moment someone changes their font size.

### Agent 3: Home Screen + Sidebar Polish

The smallest agent. Fixed the wrong keybinding hint, added a shortcuts reference table to the home screen, made the active workspace accent bar 4px (vs 3px inactive), and gave active workspaces a subtle background tint.

## Phase 2: Interaction

### Agent 4: Clipboard Support

Added the `arboard` crate for cross-platform clipboard access. The implementation has two halves:

**Copy (Ctrl+Shift+C):** Locks the terminal, reads the buffer from `topmost_line()` to `bottommost_line()` via alacritty_terminal's `bounds_to_string()`, writes to system clipboard.

**Paste (Ctrl+Shift+V):** Reads system clipboard, checks if terminal has `BRACKETED_PASTE` mode enabled (many shells use this to prevent command injection from pasted text), wraps with `\x1b[200~`/`\x1b[201~` escape sequences if so, then sends bytes to PTY.

Important: Ctrl+Shift combos must be filtered from reaching the PTY. Added a blanket filter in term_widget.rs: `if modifiers.control() && modifiers.shift() { return None; }`. Standard terminal emulator behavior -- Ctrl+Shift is the UI shortcut modifier.

### Agent 6: Scrollback UI

The key insight: alacritty_terminal already maintains a scrollback buffer internally. The grid supports `scroll_display(Scroll::Delta(lines))`. And `display_iter()` already respects the viewport offset. So rendering scrolled content requires zero changes to term_canvas.rs.

Implementation:
- Mouse wheel in term_widget.rs: convert delta to lines (`y * 3.0` for line-based, `y / cell_h` for pixel-based), call `term.scroll_display(Scroll::Delta(lines))`
- Shift+PageUp/Down via app_keys.rs subscription
- Alternate screen guard: when in ALT_SCREEN mode (vim, htop), don't scroll. Those apps handle scroll events themselves.

## Phase 3: Paper WM Identity

### Agent 7: Pane Management

Niri lets you move windows with keyboard shortcuts. We added:
- Alt+W: close focused pane
- Alt+Shift+Left/Right: reorder panes within the strip

PaperStrip gained `swap_left()` and `swap_right()` methods that swap both the pane ID and its width in parallel vectors. Focus follows the moved pane.

The Alt modifier filter in term_widget.rs changed from `modifiers.alt() && !modifiers.control() && !modifiers.shift()` to just `modifiers.alt() && !modifiers.control()`. This blocks both Alt-only AND Alt+Shift combos from reaching the PTY. Without this, Alt+Shift+Arrow would send garbage escape sequences to the terminal.

### Agent 8: Scroll Preservation + Spring Animation

Two problems solved:

**Scroll preservation:** When switching workspaces, the scroll position was lost. Added `saved_scroll_x: f32` to PaperStrip. On workspace switch: compute the focused pane's center position, save it. On switch back: restore via `scroll_to`. The computation reuses the same `compute_scroll_x` helper as the focus navigation.

**Spring animation:** Replaced the cubic ease-out `1 - (1-t)^3` with a critically-damped spring:
```
f(t) = 1 - (1 + 8t) * exp(-8t)
```

At t=0: `1 - 1*1 = 0`. At t=1: `1 - 9*exp(-8) = 0.997`. The spring decelerates more smoothly than the cubic and gives a subtle "settling" feel that matches niri's animation style. Tick rate slowed from 0.08 to 0.06 per frame for slightly longer, smoother motion.

## Phase 4: Persistence + Quality

### Agent 9: Session Recovery + CWD Tracking

Three things wired together:

**Layout persistence:** On CloseWindow, serialize all workspaces to `~/.config/cratty/layout.json`: workspace names, pane widths (ColumnWidth got Serialize/Deserialize derives), focus indices, and working directories. On startup, restore_layout() reads the file and spawns backends for each saved pane.

**CWD tracking:** The existing dead code `extract_cwd()` in terminal.rs finally got used. Wired into `process_terminal_events()`: when a title event arrives, extract the CWD from the title string and store it on the pane. This eliminates one of the two dead code warnings.

**CWD inheritance:** New panes inherit the focused pane's working directory. `add_pane_to_active_workspace()` falls back to the focused pane's `cwd` when no explicit path is given. No more "every new pane starts in home directory."

### Agent 10: Testing + CI

Created 5 test files covering the core library: column_width (resolve, adjust, same_as), view_offset (spring animation, snap behavior), config (defaults, YAML deserialization), id (monotonic counters, Display format), and focus (state transitions).

59 tests total across 6 files (including the existing paper_strip_tests).

GitHub Actions CI: fmt check, clippy, build, and test on windows-latest with cross-platform release builds for ubuntu/windows/macos.

## The 100-Line Constraint

Every source file must stay under 100 lines. This forced constant extraction:

- config.rs (148) -> config.rs (92) + config_io.rs (24)
- app_update.rs (135) -> app_update.rs (92) + app_ws_ops.rs (30), after compacting match arms
- main.rs hit exactly 100 three separate times as agents added fields and mod declarations. Required merging import lines and compacting function signatures.

The constraint is annoying in the moment but keeps the codebase navigable. No file requires scrolling to understand.

## Agent Parallelization

The key to running agents in parallel: file-level separation. Agents that touch different files can run simultaneously. When they share files (term_widget.rs was touched by Agents 1, 4, 5, and 6), they must be sequenced.

Our dependency graph:
```
Phase 1 (parallel):  [Rendering]  [Config]  [UI Polish]
Phase 2 (parallel):  [Clipboard]  [Scrollback]
Phase 3 (parallel):  [Pane Mgmt]  [Animation]
Phase 4 (parallel):  [Recovery]   [Testing]
```

Within each phase, agents target different primary files. Cross-phase, the ordering handles conflicts naturally: Phase 2 agents read files that Phase 1 agents wrote.

The biggest friction: agents that planned but needed approval to implement. The orchestrator pattern (research -> plan -> implement) adds a round-trip. For well-specified tasks, a direct implementation agent would be faster.

## What Didn't Work

**Worktree isolation:** Tried running agents in git worktrees for clean separation. Failed because the working directory (tabbyrs) isn't a git repo -- the actual project (cratty) is at a different path. Worktrees need to be created from within the repo.

**Agent 5 (mouse support) was skipped.** It was planned but depended on Agent 4 (clipboard) finishing first since both modify term_widget.rs. The clipboard agent took priority and mouse support (click-to-position, SGR mouse protocol) remains for a future session.

## Final Stats

- 4 phases, 9 agents completed (Agent 5 deferred)
- 24 new files created
- ~1500 lines added net
- 59 unit tests, all passing
- Zero compilation warnings (dead code suppressed with #[allow])
- All source files at or under 100 lines (except term_backend.rs at 112 and grid.rs at 180, both pre-existing)

## What's Next

- Mouse support (SGR mouse protocol, click-to-position for vim/htop)
- Text selection (click-drag to select, feeds into copy)
- Configurable themes (the color palette is hardcoded, config has a theme field but it's not wired)
- Quake mode (global hotkey dropdown terminal, config exists but logic is missing)
- Smooth pane width animation (currently snaps, should lerp over ~200ms)
- Drag-to-resize pane edges
- Right-click context menu on panes
