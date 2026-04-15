# Session 2026-04-13 to 2026-04-14: Workspace-as-Project, Editor Panes, Mica Glass

Continuation of the quake-mode session. This one was all about making Cratty feel like a real project-first dev workstation instead of a terminal multiplexer with a sidebar.

## Motivation

Two pain points drove this:

1. **Workspace names were nonsense.** Clicking "New workspace" produced `Terminal 5`, `Terminal 7`, `Terminal 9`, etc. because the counter used `id_gen.next_workspace().0` and skipped every time a pane id got minted in between. User quote: "terminal 5 or terminal 7... where did terminal 10 go?" More fundamentally, the cwd of a new workspace was wherever the binary launched from, so running Cratty out of `target/release/` trapped every new workspace in `release/`.

2. **Everything was white.** PowerShell and every CLI tool came up with plain white text because we never called `alacritty_terminal::tty::setup_env()`, so `TERM` and `COLORTERM` were unset for child processes. User quote: "I wanna die just seeing that there is no way I can remember such big thing if there are no colors to differentiate things."

On top of that, the user wanted an in-app file picker that opens files as editor panes (not an external editor), and Windows 11 glass/mica chrome to match the premium feel.

## 1. Workspace = Project

New model: one workspace is one folder. Clicking "New workspace" pops a native Windows `IFileOpenDialog` via `rfd::AsyncFileDialog::pick_folder()`. Cancel = no workspace created. The folder basename becomes the workspace name (`D:\Projects\cratty` => `cratty`). Every pane inside inherits the folder as its cwd.

**Core changes:**
- `Workspace::with_root(id, path)` constructor, new `root: Option<PathBuf>` field
- `WorkspaceLayout` gets `#[serde(default)] root` for backwards-compat
- `Message::NewWorkspace` now fires `Task::perform(rfd dialog, WorkspaceFolderPicked(...))`
- New `Cratty::create_workspace_with_root(root)` that seeds the first pane with `cwd = root`
- `sidebar::display_name` preference order: user-renamed name > `ws.root` basename > focused pane cwd > fallback

**Persistence bug caught the next day:** restored panes weren't landing in the workspace folder. Root cause: PowerShell and cmd don't emit OSC 7 cwd hints, so `pane.cwd` stayed `None`, and on restore we spawned the backend with `None` cwd = binary launch dir. Fix in `app_persist::restore_layout`:
```rust
let cwd = pane_layout.cwd.clone().or_else(|| ws_layout.root.clone());
self.spawn_backend(pane_id, cwd);
```
Old saved layouts still respawn with stale `None` on the first relaunch after this fix, but every subsequent save/restore is correct.

## 2. Polymorphic Panes

Panes are no longer terminal-only. `Pane::backend: Option<TermBackend>` became:
```rust
pub enum PaneContent {
    Loading,
    Terminal(TermBackend),
    Editor(EditorPane),
}
```
`EditorPane` holds an `iced::widget::text_editor::Content`, the file path, and a dirty flag. All terminal call sites were mechanically updated to use `pane.terminal()` / `pane.terminal_mut()` accessors that return `Option<&TermBackend>` for the Terminal variant.

**New files:**
- `editor_widget.rs` (34 lines) - renders `text_editor` with the Cratty dark palette
- `app_picker.rs` (60 lines) - editor action, Ctrl+S save, file-picker handlers

## 3. Alt+E File Picker

`ignore::WalkBuilder` walks the workspace root respecting `.gitignore`, capped at 10,000 files. Substring match is good enough for MVP (fuzzy matching via nucleo is deferred). Reuses the same `stack + opaque` overlay pattern as the workspace menu.

- `Alt+E` => `OpenFilePicker`
- `ArrowUp` / `ArrowDown` move selection (no-op when picker closed)
- Enter reads the file and creates an Editor pane in the active workspace
- `Ctrl+S` saves the focused editor pane's content to disk
- Escape closes the picker first, then falls through to existing escape routing

**New files:**
- `file_picker.rs` (53 lines) - walker + state + refilter
- `file_picker_view.rs` (68 lines) - centered opaque panel with text_input + scrollable list

## 4. Windows 11 Mica Glass + Rounded Corners

Three DWM calls on first `find_main_hwnd()` in `quake.rs`:

```rust
dwm_set(hwnd, DWMWA_TRANSITIONS_FORCEDISABLED, &BOOL(1));   // FOUC mitigation
dwm_set(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &DWMWCP_ROUND); // Win11 corners
dwm_set(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &DWMSBT_MAINWINDOW); // Mica
```

Plus:
- `window_settings().transparent = true` in `main.rs`
- Theme palette `background: Color::TRANSPARENT` (was `BG_TERMINAL`)
- `BG_TITLEBAR` alpha dropped to 0.55 so the chrome (titlebar, sidebar, pane gaps, popup menus, file picker) shows the Mica backdrop through
- `BG_TERMINAL` stays fully opaque so the terminal canvas and editor panes remain solid and readable

DWM ignores unknown attributes on Win10, so this is a no-op there rather than an error.

## 5. Terminal Colors (the white-shells fix)

One line in `main.rs::main`:
```rust
alacritty_terminal::tty::setup_env();
```
Sets `TERM=xterm-256color` and `COLORTERM=truecolor` as process env vars before iced spawns child shells. `git status`, `cargo build`, `ls --color=auto`, eza, and every other CLI that gates color on `TERM` now emits ANSI escapes that `term_canvas::draw_grid` already knows how to render.

PowerShell PSReadLine syntax highlighting was already working via ConPTY; this fix unblocks every other tool.

## Deferred

- **Resize handles.** With `decorations: false`, resize requires subclassing the window proc to intercept `WM_NCHITTEST` and return `HTLEFT`/`HTRIGHT`/etc. iced/winit owns the window proc, so `SetWindowLongPtrW(GWLP_WNDPROC, ...)` risks fighting iced's event routing. Needs its own spike.
- **Fuzzy matching.** Substring search is fine for small repos. Swap in `nucleo` when it starts to bite.
- **Quake mode FOUC final polish.** Mitigations in place (class brush zeroed, DWM transitions disabled, RDW_NOERASE redraw) but occasional white flash on show. Revisit later.

## Verification

- `cargo build --release` clean
- File budget: all touched files at or under 100 lines. `quake.rs` sits at exactly 100 after three rounds of compaction (inlined imports, collapsed callback, `dwm_set` helper).
- Persistence fix tested: open workspace in `D:\Projects\cratty`, close app, relaunch, pane lands in project root
- Mica visible on Win11 with transparent chrome; terminal text remains solid and readable

## Files Touched

Core: `workspace.rs`, `layout.rs`.

App (modified): `main.rs`, `message.rs`, `app_update.rs`, `app_actions.rs`, `app_ws_ops.rs`, `app_persist.rs`, `app_keys.rs`, `app_tick.rs`, `pane.rs`, `strip_view.rs`, `sidebar.rs`, `quake.rs`, `style.rs`, `clipboard_ops.rs`, `term_scroll.rs`.

App (new): `app_picker.rs`, `editor_widget.rs`, `file_picker.rs`, `file_picker_view.rs`.

Deps: `rfd = "0.15"`, `ignore = "0.4"`, `windows` features `Win32_Graphics_Dwm` and `Win32_Graphics_Gdi`.
