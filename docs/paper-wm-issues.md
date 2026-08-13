# Paper Window Manager - Known Issues & TODO

Tracking issues discovered during paper WM implementation.

## Open Issues

### Visual

- [ ] Sidebar has no resize handle - width is hardcoded at 200px
- [ ] No visual indicator of which direction you can scroll (no peek of adjacent panes)

### Behavior

- [ ] Mouse support: click-to-position (SGR mouse protocol) not implemented
- [ ] Text selection: click-drag to select text not implemented
- [ ] Pane width changes snap instantly (should animate/lerp over ~200ms)
- [ ] Wispr Flow / STT tool support: test with Wispr Flow on Windows (Ctrl+V paste added, needs real-device validation)
- [ ] No drag-to-resize on pane edges
- [ ] No right-click context menu on panes
- [ ] Quake mode: visual position/size (slide-down dropdown style) not implemented; only window show/hide via global hotkey

### Architecture

- [ ] term_backend.rs at 112 lines (PTY wrapper, hard to split further)
- [ ] Terminal resize feedback loop: never use terminal layout size for viewport width calculation (see iced-patterns.md)

## Completed

- [x] Basic sidebar with workspace list
- [x] Titlebar simplified to window chrome only
- [x] Pane dividers between terminals
- [x] Focus indicator (blue border + dim overlay on unfocused)
- [x] Replaced iced_term with own Canvas-based renderer (PR #10)
- [x] term_backend.rs wrapping alacritty_terminal directly
- [x] term_canvas.rs drawing grid cells on iced Canvas
- [x] term_widget.rs Canvas Program with keyboard input
- [x] Dropped iced_term dependency
- [x] iced scrollable compression investigated (correct by design, closed iced-rs/iced#3299)
- [x] iced_term coordinate bugs identified (Size::ZERO intrinsic + absolute coords in local frame)
- [x] Viewport width feedback loop bug found and documented
- [x] True paper WM: horizontal scrollable with per-pane widths
- [x] Niri-style preset width cycling (1/3, 1/2, 2/3) + maximize toggle
- [x] Alt+key shortcuts (T, N, B, R, F, W, Left, Right, Minus, Equal)
- [x] Proper terminal input (ANSI escape sequences for named keys, Ctrl+letter)
- [x] Space bar fix (Named::Space in iced)
- [x] 256-color xterm palette (color cube + grayscale ramp)
- [x] Text attributes: bold, dim, italic, underline, strikethrough, inverse
- [x] Cursor shapes: filled block (focused) and hollow outline (unfocused)
- [x] Dynamic font metrics from config (replaced hardcoded CELL_W/CELL_H)
- [x] Config integration: AppConfig loaded on startup, shell profiles used for PTY
- [x] Home screen shows correct shortcuts (Alt+T) with reference table
- [x] Sidebar polish: active workspace background, accent bar width, brighter header
- [x] Clipboard: Ctrl+Shift+C/V with bracketed paste support
- [x] Mouse wheel scrollback (3 lines per notch, alternate screen guard)
- [x] Shift+PageUp/Down keyboard scrollback
- [x] Close focused pane (Alt+W)
- [x] Reorder panes within strip (Alt+Shift+Left/Right)
- [x] Workspace scroll position preserved across switches
- [x] Scroll animation: replaced broken spring with ease-out-quart, wired ViewOffset to drive scrollable each tick
- [x] Sidebar "New workspace" button: icon used Length::Fill causing 50/50 split with text, fixed to Shrink
- [x] Workspace context menu stuck open: HideWsMenu was never sent, added toggle + clear on delete/switch
- [x] Ctrl+V paste in terminal: intercept in canvas before raw byte reaches PTY (enables Wispr Flow STT support)
- [x] Theme: Cratty Warm Dark (Zed One Dark base, warmed to purple-grey with #61afef accent)
- [x] Custom iced theme palette so empty space matches content background
- [x] ANSI 16-color palette ported to Zed One Dark
- [x] Pane border visibility: BG_TITLEBAR strip background contrasts with BG_TERMINAL panes
- [x] Workspace context menu polish: Zed-style padding (tight outer, roomy inner items)
- [x] Color submenu with caret-right icon (Phosphor U+E13A) and click-toggle
- [x] Layer-based popup system: opaque() wrapper for proper click+hover blocking
- [x] Backdrop click dismissal that doesn't break menu interaction
- [x] Aceternity-style soft shadows: 10% opacity, 25px blur, subtle white border
- [x] Quake mode global hotkey: Ctrl+` via global-hotkey crate, configurable in config.yaml
- [x] new modules: layers.rs (compose), quake.rs (global hotkey)
- [x] Session recovery: layout.json saved on close, restored on startup
- [x] CWD tracking from terminal title events + inheritance for new panes
- [x] 59 unit tests across 6 test files
- [x] GitHub Actions CI (fmt, clippy, build, test, cross-platform)
- [x] All source files at or under 100 lines
- [x] config.rs split to comply with line limit
- [x] Ctrl+Shift combos filtered from reaching PTY
- [x] main.rs decomposed from ~290 lines to 100 lines across 8 modules
