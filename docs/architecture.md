# Cratty Architecture

## What Is Cratty?

Niri for terminals. A native terminal emulator with a paper window manager where each pane defines its own width and the viewport scrolls horizontally. Built in Rust with iced 0.14.

## Stack

```
cratty.exe (~3MB native binary)
  |
  iced 0.14 (Elm architecture, wgpu GPU rendering)
  |-- Custom chrome (decorations: false, Phosphor Bold icons)
  |-- Sidebar (workspace list, rename, color badges)
  |-- Horizontal scrollable (paper WM strip)
  |     |-- Per-pane containers with fixed pixel widths
  |     |-- Canvas widgets (terminal rendering)
  |
  alacritty_terminal 0.25.1 (VT parser + grid + scrollback)
  |-- Term<EventProxy> per pane
  |-- EventLoop per PTY (spawned thread)
  |-- Notifier for input delivery
  |
  Shell (pwsh.exe / bash / cmd.exe via config or auto-detect)
```

## Workspace Crates

### cratty_app (binary, ~2900 lines across 24 modules)

All GUI code. Modules organized by concern:

**App lifecycle:** main.rs (state + iced bootstrap), app_update.rs (message dispatch), app_tick.rs (per-frame updates), app_keys.rs (keyboard subscriptions), app_actions.rs (workspace/pane operations), app_ws_ops.rs (rename/menu handlers), app_persist.rs (layout save/restore)

**Terminal layer:** term_backend.rs (PTY wrapper), term_canvas.rs (grid cell rendering with flags), term_colors.rs (color resolution), term_palette.rs (256-color xterm table), term_decor.rs (underline/strikethrough), term_cursor.rs (block/hollow cursor), term_widget.rs (Canvas Program + input), term_input.rs (key-to-ANSI conversion), term_scroll.rs (scrollback helper)

**UI:** strip_view.rs (paper strip scrollable), sidebar.rs (workspace list), ws_item.rs (workspace entry), ws_menu.rs (context menu), titlebar.rs (window chrome), home.rs (empty state), widgets.rs (icon helpers), style.rs (color constants)

**Clipboard:** clipboard.rs (arboard wrapper), clipboard_ops.rs (copy/paste logic)

### cratty_core (library, ~600 lines)

Domain types and config:
- `AppConfig` + `config_io` -- YAML config with font, shell profiles, quake mode
- `PaperStrip` -- horizontal pane layout with per-pane widths, focus, swap, maximize
- `ColumnWidth` -- Proportion(f32) or Fixed(f32), serializable
- `ViewOffset` -- spring-based animation state
- `FontMetrics` -- cell dimensions derived from font size
- `Workspace`, `PaneId`, `WorkspaceId`, `IdGen` -- identity types
- `FocusState`, `InputMode` -- keyboard focus state machine
- `SavedLayout` -- serializable workspace layout for session recovery

### cratty_vt (Zig VT parser, dormant)

A Zig-based VT parser and terminal grid with Rust FFI bindings. Built but unused since alacritty_terminal handles VT parsing. Kept for the planned Zig rewrite.

## Data Flow

```
Shell (stdout bytes)
  |
  v
alacritty_terminal EventLoop (background thread, parses VT sequences)
  |
  v
EventProxy -> mpsc::channel -> event_rx (Title, Exit events)
  |
  v
app_tick.rs: drain_events() on each Tick (16ms / 60fps)
  |-- Event::Title -> update pane.title + extract CWD
  |-- Event::Exit -> remove pane, clean empty workspaces
  v
term_canvas.rs: draw_grid() locks Term, iterates grid.display_iter()
  |-- Per cell: read flags (bold/dim/italic/underline/inverse/hidden)
  |-- Resolve colors via term_palette (256 indexed + RGB)
  |-- Draw text with Font { weight, style } on Canvas frame
  |-- Draw decorations (underline, strikethrough) via term_decor
  |-- Draw cursor (filled block or hollow outline) via term_cursor
  v
wgpu -> GPU -> pixels

Keyboard input (reverse path):
  iced KeyPressed event -> term_widget.rs update()
  |-- Filter: Alt combos -> app shortcuts (never reach PTY)
  |-- Filter: Ctrl+Shift combos -> clipboard/UI shortcuts
  |-- term_input::key_to_bytes() -> ANSI escape sequences
  |-- Notifier.notify(bytes) -> PTY stdin
```

## Paper WM Architecture

The core philosophy: each pane defines its own width. The viewport scrolls.

```
Workspace "Dev"
  PaperStrip
    panes:  [PaneId(1), PaneId(2), PaneId(3)]
    widths: [Proportion(0.5), Proportion(0.333), Proportion(0.667)]
    focus_idx: 1  (PaneId(2) is focused)
    saved_scroll_x: 450.0  (preserved across workspace switches)
```

Width resolution: `Proportion(0.5)` at viewport 1200px = 600px. `Fixed(400.0)` = 400px always.

Preset cycling (Alt+R): 1/3 -> 1/2 -> 2/3 -> 1/3. Maximize (Alt+F) saves previous width and sets Proportion(1.0). Incremental resize (Alt+Minus/Equal) adjusts by 0.1.

Strip rendered as iced `scrollable(row(...)).direction(Horizontal)` with invisible scrollbar. Each pane gets `Length::Fixed(pane_w)`. Focused pane centered via `operation::scroll_to`.

## Key Design Decisions

### Why custom Canvas renderer instead of iced_term?

iced_term's TerminalView doesn't render inside a horizontal scrollable with fixed pixel widths (goes blank). Canvas widgets use local coordinates with `with_translation`, which iced's scrollable handles correctly. Building our own renderer also gives us control over text attributes, cursor shapes, and scroll behavior.

### Why Alt modifier for shortcuts?

Super/Win key is intercepted by Windows OS (Super+T opens taskbar, Super+R opens Run dialog). Ctrl+Shift+N was awkward to press and leaked ^N into the terminal. Alt is comfortable, conflict-free for the specific keys chosen (T, N, B, R, F, W, Left, Right, Minus, Equal), and matches the convention of many Linux window managers.

### Why parallel Vec<PaneId> + Vec<ColumnWidth>?

Each pane needs its own width, but PaperStrip stores pane IDs (not owned Pane objects). A parallel widths vector keeps width data co-located with the strip layout. Both vectors are always modified together (push, remove, swap).

### Why spring animation?

Cubic ease-out feels mechanical. Critically damped spring `1 - (1+8t)*exp(-8t)` matches niri's settling feel. At 60fps with 0.06 step per frame, animations take ~270ms.

### Why session recovery via JSON?

Workspaces, pane widths, focus indices, and CWDs are serialized to `~/.config/cratty/layout.json` on close. On startup, the layout is restored and new PTY backends are spawned for each saved pane. serde_json keeps it simple and human-readable.

## Comparison

|                  | Tabby          | Cratty            |
|------------------|----------------|-------------------|
| Binary size      | ~200MB         | ~3MB              |
| Runtime          | Electron       | Native (wgpu)     |
| Language         | TypeScript     | Rust              |
| Memory           | ~300MB+        | ~30MB             |
| Terminal engine  | xterm.js       | alacritty_terminal|
| GUI framework    | Chromium       | iced 0.14         |
| Layout model     | Tiling splits  | Paper WM          |
| Architecture     | Component-based| Elm (functional)  |
