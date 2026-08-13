# Popup And Dialog System Design

## Goal

Give Cratty one coherent popup language instead of a collection of unrelated transient surfaces.

The target blend is:

- Zed for restraint, hierarchy, and editor seriousness
- Aceternity for spacing discipline, composition, and soft surface polish

This should improve the file picker, workspace menu, hover popup, and toast system without making the app feel flashy or overdesigned.

## Design Rules

- Use one shared material family for popup-like UI.
- Keep chrome quiet: low-contrast borders, soft elevation, and deliberate spacing.
- Avoid nested framed boxes when one surface plus internal spacing is enough.
- Prefer stronger composition over stronger decoration.
- Keep dialogs and menus visually related, but let density vary by purpose.

## Surface Classes

### Dialog

Used for the file picker and future centered modal panels.

- Largest padding and strongest internal structure
- Soft rounded corners
- Light border and calm shadow
- Backdrop tint strong enough to isolate the surface without dimming the whole app too aggressively

### Menu

Used for workspace menus and small contextual panels.

- Same material family as dialogs
- Tighter density
- Shorter row height
- Minimal separation between items

### Tooltip

Used for hover popups and compact explanatory surfaces.

- Same palette and border language
- Smaller radius and padding
- No heavy shadow or framing
- Reads as a micro-surface, not a dialog

### Toast

Used for transient success, info, and error notifications.

- Same outer shell as the other surfaces
- Accent is carried by a slim border or edge emphasis, not loud fill colors
- Bottom-right placement remains unchanged

## Implementation Shape

- Add shared popup tokens to the style layer:
  - surface background variants
  - soft border color
  - shadow color
  - standard radii
- Add shared surface helpers in `widgets.rs` for:
  - panel/dialog surface style
  - menu surface style
  - tooltip surface style
  - toast surface style
- Apply the new surface system to:
  - `file_picker_view.rs`
  - `ws_menu.rs`
  - `editor_widget.rs` hover popup
  - `widgets.rs` toast rendering

## Expected Outcome

- The file picker becomes the visual anchor for Cratty’s transient UI.
- Workspace menus feel like compact siblings of the picker instead of separate styling experiments.
- Hover popups stop looking like ad hoc overlays.
- Toasts feel authored by the same app.
- The app stays simple and clean, but more intentional in spacing and surface polish.
