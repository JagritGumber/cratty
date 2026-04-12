# Layer Architecture: Implementation Plan

## Problem

The app mixes base UI and popups in ad-hoc ways. Hover events bleed through overlays, menu positioning is fragile, and there's no consistent pattern for popup rendering. The code should be explicitly structured around layers so that any developer can see what's on which layer and how events flow.

## Key Discovery

iced's `stack` widget ALREADY supports cursor levitation (blocking hover on lower layers). The fix is one method call: `mouse_area(...).interaction(mouse::Interaction::Idle)`. This makes the stack's event loop call `cursor.levitate()` for all layers below the backdrop, preventing hover state from reaching them.

iced also has a proper `overlay` system (used by PickList for dropdowns) that renders above the widget tree and handles events before the main content. For complex popups, this is the correct primitive.

## Architecture Overview

### Layer Definitions

```
Layer 0 (Base):     Titlebar + Sidebar + Content
Layer 1 (Popups):   Backdrop + Positioned Menus
Layer 2 (Tooltips): Future - hover hints, status info
```

### Code Structure

```
main.rs::view()
  |
  +-- layer_base()     -> Element    (layer 0)
  |   +-- titlebar::view_titlebar()
  |   +-- sidebar::view_sidebar()
  |   +-- strip_view::view_strip() | home::view_home()
  |
  +-- layer_popups()   -> Option<Element>  (layer 1)
  |   +-- backdrop (mouse_area with interaction set)
  |   +-- popup::render_popup(popup_state)
  |       +-- ws_menu, color submenu, future popups
  |
  +-- compose: stack![layer_0, layer_1?]
```

### File Layout After Refactor

```
src/
  main.rs           -- app struct, layer composition in view()
  layers.rs         -- compose_layers() + Layer enum/helpers
  popup.rs          -- PopupState, render_popup(), backdrop
  sidebar.rs        -- workspace list (layer 0 only, no popups)
  ws_item.rs        -- workspace item rendering
  ws_menu.rs        -- menu content (what's inside the popup)
  strip_view.rs     -- terminal pane strip
  home.rs           -- empty state home screen
  titlebar.rs       -- window chrome
  ...
```

## Phase 1: Fix Event Blocking (Quick Win)

**Files:** `app_ws_ops.rs`

The backdrop `mouse_area` needs `.interaction(mouse::Interaction::Idle)` so the stack levitates the cursor for lower layers.

```rust
// Before
let backdrop = mouse_area(
    container(Space::new()).width(Length::Fill).height(Length::Fill),
)
.on_press(Message::HideWsMenu)
.into();

// After
let backdrop = mouse_area(
    container(Space::new()).width(Length::Fill).height(Length::Fill),
)
.on_press(Message::HideWsMenu)
.interaction(mouse::Interaction::Idle)
.into();
```

This single line fix makes iced's stack properly block all hover events to layer 0 when the popup layer is active. Remove all `menu_open` band-aid flags from `ws_item.rs`, `sidebar.rs`, etc.

**Verification:** Open menu, hover over workspace items behind it. They should NOT show hover effects.

## Phase 2: Extract Popup System

**New file:** `popup.rs`

Move all popup state management and rendering out of `app_ws_ops.rs` into a dedicated module.

```rust
// popup.rs

use iced::widget::{container, mouse_area, stack, Space};
use iced::mouse;
use iced::{Element, Length, Point};

use crate::message::Message;

/// Render a popup overlay with backdrop
pub fn render_popup<'a>(
    content: Element<'a, Message>,
    position: Point,
    dismiss_msg: Message,
) -> Element<'a, Message> {
    let backdrop = mouse_area(
        container(Space::new()).width(Length::Fill).height(Length::Fill),
    )
    .on_press(dismiss_msg)
    .interaction(mouse::Interaction::Idle);

    let popup = container(content)
        .padding(iced::Padding {
            top: position.y, left: position.x,
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fill);

    stack![backdrop, popup]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
```

This is reusable for any popup: workspace menu, color submenu, future rename dialog, settings panel, etc.

**Files changed:**
- NEW: `popup.rs`
- `app_ws_ops.rs`: `menu_overlay()` calls `popup::render_popup()`
- `main.rs`: add `mod popup;`

## Phase 3: Extract Layer Composition

**New file:** `layers.rs`

The main view function should read as a layer list, not nested conditionals.

```rust
// layers.rs

use iced::widget::stack;
use iced::{Element, Length};
use crate::message::Message;

/// Compose layers into a single element. Layer 0 is base, higher = on top.
pub fn compose(layers: Vec<Element<'_, Message>>) -> Element<'_, Message> {
    if layers.len() == 1 {
        return layers.into_iter().next().unwrap();
    }
    stack(layers).width(Length::Fill).height(Length::Fill).into()
}
```

Main view becomes:

```rust
// main.rs view()
fn view(&self) -> Element<'_, Message> {
    let mut layers = vec![self.view_base()];
    if let Some(popup) = self.view_popups() {
        layers.push(popup);
    }
    layers::compose(layers)
}

fn view_base(&self) -> Element<'_, Message> {
    // titlebar + sidebar + content (current body logic)
}

fn view_popups(&self) -> Option<Element<'_, Message>> {
    // menu_overlay logic using popup::render_popup
}
```

**Files changed:**
- NEW: `layers.rs`
- `main.rs`: restructure `view()` into `view_base()` + `view_popups()` + `layers::compose()`

## Phase 4: Clean Up Band-aids

Remove all `menu_open` flags that were added as workarounds:

- `ws_item.rs`: Remove `menu_open` parameter and hover suppression logic
- `sidebar.rs`: Remove `menu_open` pass-through to `new_ws_button()` and `view_ws_item()`
- `sidebar.rs`: `new_ws_button()` removes `menu_open` parameter

The `.interaction(Idle)` on the backdrop handles everything.

**Files changed:**
- `ws_item.rs`: simplify signature and style
- `sidebar.rs`: simplify signature and new_ws_button

## Phase 5: Sublayer System (Shadow + Content)

The menu container currently applies shadow inline via `container::Style::shadow`. This is fine for now but the design doc calls for sublayers.

For the shadow sublayer concept: iced's `container::Style::shadow` already renders shadows BELOW the container content, which is exactly the sublayer behavior described. No custom rendering needed -- iced handles this natively.

Document this as "sublayers are handled by iced's container shadow/border system, no custom implementation required."

## Phase 6: Future Popup Types

Once `popup.rs` exists, adding new popups is trivial:

```rust
// Example: rename dialog
let rename_dialog = popup::render_popup(
    rename_dialog_content(ws_id, &self.rename_text),
    Point::new(x, y),
    Message::CancelRename,
);
```

Each popup type just provides content + position + dismiss message.

## File Size Constraints

All files must stay under 100 lines. Estimated sizes after refactor:

| File | Current | After | Notes |
|------|---------|-------|-------|
| main.rs | 105 | ~95 | view() split into view_base/view_popups |
| layers.rs | NEW | ~15 | compose() helper |
| popup.rs | NEW | ~35 | render_popup() + backdrop |
| app_ws_ops.rs | 65 | ~40 | menu_overlay delegates to popup.rs |
| ws_item.rs | 84 | ~75 | remove menu_open param |
| sidebar.rs | 95 | ~90 | remove menu_open pass-through |
| ws_menu.rs | 96 | ~96 | unchanged |

## Execution Order

1. Phase 1 first (one-line fix, immediate UX improvement)
2. Phase 4 immediately after (remove band-aids that Phase 1 makes unnecessary)
3. Phase 2 + 3 together (extract popup.rs and layers.rs)
4. Phase 5 is documentation only
5. Phase 6 is future work, no code now

## Verification

After each phase:
1. `cargo check` -- no warnings
2. `cargo build --release` -- run the app
3. Open workspace menu: hovers ONLY on menu items, not on sidebar behind
4. Click outside menu: dismisses
5. Click Rename: triggers rename, not workspace switch
6. Click Color: toggles submenu
7. Click color swatch: applies color, closes menu
8. Escape: dismisses everything
9. All files under 100 lines
