# Session 2026-04-11: Theme System, Layer Architecture, Quake Mode

Continuation of the 2026-04-09 session. Started with theme work, descended into a complete rewrite of the popup system, and ended with global hotkey support.

## 1. Theme Alignment with Zed One Dark

User wanted to align colors with Zed editor for "comfy" feel. Researched Zed's One Dark palette from `zed-industries/zed/assets/themes/one/one.json`.

**Initial port (exact Zed values):**
- Chrome `#2f343e`, Content `#282c34`, Accent `#61afef` (blue)
- Off-white text `#dce0e5` instead of pure white
- Updated all 16 ANSI colors to Zed One Dark palette
- Custom iced theme palette so empty space matches the warm content color

**Files changed:** `style.rs`, `term_colors.rs`, `strip_view.rs`, `home.rs`, `titlebar.rs`, `ws_menu.rs`, `term_cursor.rs`

User loved it but wanted to differentiate from Zed.

## 2. Cratty Warm Dark Variant

User wanted warmer tones (purple/mauve undertone) since Zed's pure blue-gray is the outlier among dark themes (Dracula, Gruvbox, Catppuccin all lean warm).

**Final palette:**
| Surface | Hex | RGB |
|---|---|---|
| Chrome (sidebar/titlebar) | `#33303a` | (51, 48, 58) |
| Content (terminal) | `#2a2732` | (42, 39, 50) |
| Menu hover | `#3a3544` | (58, 53, 68) |
| Selected | `#433e4e` | (67, 62, 78) |
| Text active | `#e2dde7` | (226, 221, 231) |
| Text inactive | `#b2aabd` | (178, 170, 189) |
| Text dim | `#706682` | (112, 102, 130) |
| Border muted | `#4e485a` | (78, 72, 90) |
| Accent (blue) | `#61afef` | (97, 175, 239) -- kept Zed's blue |

Blue accent pops nicely against warm purple-grey backgrounds (complementary contrast).

## 3. Empty Space Color Fix

User noticed empty space was iced's default cold dark gray, not the warm content color.

**Root cause:** `theme()` returned `Theme::Dark` (built-in iced theme).

**Fix:** Custom iced theme palette using `Theme::custom("Cratty", Palette { background: BG_TERMINAL, ... })`. Added a `cratty_theme()` function in style.rs.

## 4. Pane Border Visibility

User: "the very first pane doesn't have any border so hard to tell how much of it is empty space if you only have one pane".

**Root cause:** Default pane width is 50% (`Proportion(0.5)`), so a single pane only fills half the screen. The border was there but the empty space behind it shared the same color, making the pane invisible against the background.

**Fix:** Made the strip area background `BG_TITLEBAR` (lighter chrome) while panes use `BG_TERMINAL` (darker content). Added explicit `BG_TERMINAL` background to pane container with a 2px border radius. Now the contrast between pane and surrounding space is clearly visible, and the focused-pane accent border has something to frame.

**Files:** `strip_view.rs`

## 5. Workspace Menu Polish (Phase 1)

User: "zed popup menus have a lot less padding for the outer shell and rather have more padding for the inner item".

**Changes:**
- Outer column: `padding(8) -> padding(4)`, spacing `4 -> 2`
- Inner buttons: `padding([4, 8]) -> padding([6, 12])`
- Container: removed border, added Aceternity-style soft shadow

## 6. Color Submenu with Caret Icon

User wanted "Color" to look like Rename/Delete with a chevron, opening a submenu on interaction.

**Flow rewrite:**
- Removed the inline color swatches row
- Added "Color" as a regular menu item with `ICO_CARET_RIGHT` (Phosphor Bold `\u{E13A}`, found by checking the official Phosphor CSS file)
- Added `ToggleColorSubmenu` message and `color_submenu: bool` state
- Color submenu rendered as a separate floating panel

**Files:** `ws_menu.rs`, `widgets.rs`, `message.rs`, `app_update.rs`, `main.rs`

## 7. Layer-Based Popup System (the big refactor)

User got frustrated: "I want all the components defined in layers. Every part of this application has layers... when my mouse is on some object that is obstructing Z zero, then Z zero shouldn't be interactable."

The previous approach was riddled with bugs:
- **Click-through**: Clicking menu items also fired the workspace switch behind
- **Hover bleed**: Workspace items showed hover state when menu was on top
- **Layout shift**: Menu was inside the sidebar column, pushing content down
- **Wrong anchor**: Menu appeared at wrong position relative to trigger

### Research Phase

Three parallel research agents:

1. **iced community patterns**: Found the official iced modal example uses `stack! + opaque() + mouse_area`. The `opaque()` function is the missing piece.

2. **Cross-framework comparison**: Researched Flutter (ModalBarrier), GTK (input grab), WPF (AdornerLayer + IsHitTestVisible), egui (paint order), Web (event.stopPropagation), SwiftUI (overlay modifier), Slint. Flutter's ModalBarrier pattern is closest to iced's `opaque()`.

3. **iced internals deep dive**: Read `iced_core/src/overlay/nested.rs` and `iced_widget/src/stack.rs`. Discovered that iced's stack already supports cursor levitation: when a top layer's `mouse_interaction()` returns non-None, lower layers receive `Cursor::Levitating` which makes their `cursor.is_over()` return false.

### The Discovery: `opaque()`

`iced::widget::opaque(content)` is a wrapper widget defined in `iced_widget/src/helpers.rs:577`. It does TWO things:

1. **Lines 681-683**: On mouse press, if cursor is over bounds, calls `shell.capture_event()` -- blocks click propagation
2. **Lines 699-705**: Returns `mouse::Interaction::Idle` when cursor is over bounds -- triggers the stack's `cursor.levitate()`, blocking hover on lower layers

This is exactly the layer behavior we needed, built into iced.

### Architecture

```
main.rs::view()
  |
  +-- view_base()      -> Element  (Layer 0)
  |   +-- titlebar + sidebar + content
  |
  +-- view_popups()    -> Option<Element>  (Layer 1+)
  |   +-- backdrop (mouse_area, dismisses on click)
  |   +-- positioned(opaque(menu_content), x, y)
  |   +-- positioned(opaque(submenu), x', y')  -- if submenu open
  |
  +-- layers::compose(vec![layer_0, layer_1?])
```

New files:
- `layers.rs` (11 lines): `compose(layers)` helper using iced's `stack`
- `popup.rs` (35 lines): `render_popup()` reusable popup wrapper

Files changed:
- `main.rs`: Split `view()` into `view_base()` + `view_popups()` + `layers::compose()`
- `app_ws_ops.rs`: `menu_overlay()` returns the popup layer
- `ws_item.rs`: Removed `menu_open` band-aid flag
- `sidebar.rs`: Removed `menu_open` pass-through

### The `opaque()` Wrapping Order Trap

First attempt:
```rust
opaque(positioned(content, x, y))  // WRONG: full-screen opaque
```

This made the entire screen opaque on the popup layer, blocking ALL events to lower layers. User reported: "I am unable to interact with anything other than that color one. I can't even click outside to close it."

Fix:
```rust
positioned(opaque(content), x, y)  // RIGHT: only content is opaque
```

Now `opaque()` only covers the actual menu pixels. The transparent positioning container lets clicks pass through to the backdrop or to other menus on the same layer.

### Backdrop Click Dismiss Bug

Initial backdrop was wrapped in `opaque()` too. The `opaque()` captured the click before `mouse_area.on_press` could process it. Fix: don't wrap the backdrop in `opaque()`. Plain `mouse_area` catches dismiss clicks; only the menu content needs to be opaque.

### Final Layer Behavior

- Click on color swatches -> hits opaque region of layer 2 -> handled
- Click on main menu -> layer 2 transparent there -> falls to layer 1 opaque -> handled
- Click on empty space -> both layers transparent -> hits backdrop -> dismisses
- Hover on workspace items below menu -> menu's `Interaction::Idle` levitates cursor -> workspace items see `Cursor::Levitating` -> hover state suppressed

## 8. Aceternity-Style Soft Shadows

User: "shadows are too harsh, can we see how acceternity UI does it?"

Researched Aceternity's navbar-menu component. Their dark dropdown uses:
- `shadow-xl` = soft diffused shadow
- `border-white/[0.2]` = subtle 20% white edge
- `rounded-2xl` = generous radius

**Cratty values:**
| Property | Before | After |
|---|---|---|
| Shadow opacity | `0.5` | `0.1` |
| Shadow offset | `(0, 4)` | `(0, 10)` |
| Shadow blur | `12px` | `25px` |
| Border | none | `white 8% opacity, 1px` |
| Border radius | `6px` | `8px` |

## 9. Quake Mode Hotkey

User: "Let's work on quake mode hotkey first, make it idk alt + space etc, some app takes something we gotta remove many I guess"

Researched hotkey conflicts:
| Hotkey | Conflict |
|---|---|
| Ctrl+Space | Wispr Flow, VS Code IntelliSense, Zed autocomplete |
| Alt+Space | Windows system menu, PowerToys Run |
| Win+\` | Windows Terminal, PowerToys FancyZones |
| Ctrl+\` | VS Code (but moot since Cratty replaces it) |
| F12 | Browser DevTools, but used by Guake/Yakuake on Linux |

**Default chosen**: `Ctrl+\`` -- same as Warp, intuitive (backtick = console).

**Implementation:**
- Added `global-hotkey = "0.6"` (Tauri team's crate)
- Created `quake.rs` with `QuakeHotkey::register()` and a subscription
- Hotkey parser supports Ctrl, Alt, Shift, Super modifiers + backtick/F12/F11/Space keys
- Subscription polls `GlobalHotKeyEvent::receiver()` every 50ms, emits `Message::ToggleQuake`
- `ToggleQuake` handler uses `window::is_minimized` then minimize/restore + gain_focus
- Config-driven: reads `quake_mode.enabled` and `quake_mode.hotkey` from `AppConfig`

**Files:** `quake.rs` (new), `main.rs`, `app_update.rs`, `message.rs`, `config.rs`, `Cargo.toml` (workspace + app)

## Notable Pitfalls

### iced 0.14 Subscription API
The old `iced::subscription::channel()` was renamed. New API:
```rust
Subscription::run(|| iced::stream::channel(10, |mut tx: Sender<Message>| async move { ... }))
```
The closure parameter needs an explicit type annotation because Rust can't infer it from `tx.try_send(Message::...)`.

### Window Visibility
iced 0.14 doesn't have `window::toggle_visibility`. Used the minimize/restore pattern instead with `window::is_minimized().then(...)` for the conditional.

### Module Organization
The `popup.rs` module is currently unused (the inline implementation in `app_ws_ops.rs:menu_overlay` was simpler). Removed from `mod` declaration but file kept for future popup additions.

## Final State

- All files at or under 100 lines
- 68 unit tests passing
- Layer-based popup system with proper click and hover blocking
- Cratty Warm Dark theme with Aceternity-style shadows
- Global Ctrl+\` quake mode toggle
- Two new modules: `layers.rs`, `quake.rs`

## Key Learnings

1. **Read framework source code first.** The `opaque()` function existed in iced all along. Hours of frustration could have been avoided by reading `iced_widget/src/helpers.rs` earlier.

2. **Cursor levitation is iced's official mechanism for layer event blocking.** Not custom widgets, not Widget::overlay() (that's for overlay-as-popup, different concept).

3. **Wrap order matters with `opaque()`.** `opaque(positioned(x))` makes the whole screen opaque. `positioned(opaque(x))` makes only the visible content opaque.

4. **`global-hotkey` crate is the Tauri team's standard.** Cleaner API than livesplit-hotkey for desktop apps.

5. **Subscription channel APIs change between iced versions.** The 0.14 form requires explicit Sender type annotation.
