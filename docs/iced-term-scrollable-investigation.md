# iced_term Scrollable Rendering Investigation

## Problem

TerminalView renders blank/gray when placed inside an iced horizontal scrollable with fixed pixel widths. Works fine with Length::Fill or FillPortion outside a scrollable.

## What Works

- Single terminal with `Length::Fill` -- always works
- Multiple terminals with `Length::FillPortion(1)` in a `row` -- works (no scrollable)
- This is because FillPortion resolves to concrete pixels via iced's layout engine

## What Doesn't Work

- Any terminal inside `scrollable(row![...]).direction(Horizontal)` with fixed pixel widths
- Terminals get correct layout dimensions (confirmed via debug) but render nothing

## Investigation Timeline

### Attempt 1: Suspected iced's scrollable passes f32::INFINITY

We initially blamed iced's scrollable for passing `f32::INFINITY` as max width to children. Filed iced-rs/iced#3299.

**Finding:** This was WRONG. iced's scrollable uses a "compression" system:
- Sets max width to `f32::INFINITY` for the scroll axis
- Sets `compression = true` for Fill children
- When compression is true, `limits.resolve(Fill)` falls to the intrinsic size fallback
- This is intentional design -- the scrollable needs to know children's actual content size

**Closed the issue.** The behavior is correct.

### Attempt 2: Fixed iced_term's Size::ZERO intrinsic fallback

TerminalView's `layout()` passes `Size::ZERO` as intrinsic size:
```rust
let size = limits.resolve(Length::Fill, Length::Fill, Size::ZERO);
```

When compression is active (inside scrollable), Fill resolves to intrinsic size = `Size::ZERO` = 0x0 dimensions.

**Fix applied:**
```rust
let cell = &self.term.font.measure;
let intrinsic = Size::new(80.0 * cell.width, 24.0 * cell.height);
let size = limits.resolve(Length::Fill, Length::Fill, intrinsic);
```

**Result:** Layout debug showed real dimensions (656x436 intrinsic, resolving to viewport width when compression is false). But terminals STILL rendered blank.

**Finding:** The intrinsic size fix is necessary but not sufficient. Layout gives correct sizes, but the rendering pipeline still fails.

### Attempt 3: Fixed draw() coordinate system

TerminalView's `draw()` method has two issues:

**Issue A: Frame sized to viewport, not layout bounds**
```rust
// Line 454 - creates frame sized to scrollable's visible window
let geom = self.term.cache.draw(renderer, viewport.size(), |frame| { ... });
```

Inside a scrollable, `viewport` is the scrollable's visible rectangle, not the terminal's bounds. The frame is too small to contain cells drawn at absolute positions.

**Issue B: Cells drawn at absolute screen coordinates**
```rust
// Lines 451-452 - absolute position from layout
let layout_offset_x = layout.position().x;
let layout_offset_y = layout.position().y;

// Line 476 - cells at absolute coordinates in a local frame
let x = layout_offset_x + (col * cell_width);
```

The cache frame has local coordinates (0,0 to width,height). But `layout.position()` returns the widget's absolute screen position. When a terminal is at x=1200 (second pane in scrollable), cells are drawn at x=1200+ which exceeds the frame's bounds.

**Fix attempted:** Drew cells at (0,0) local coordinates, used `with_translation(bounds.x, bounds.y)` to position geometry (matching iced's own Canvas widget pattern).

**Result:** Still blank. Single pane also broke.

### Attempt 4: Deeper analysis of draw_geometry + cache interaction

**Finding:** The geometry cache in iced_term (`self.term.cache.draw()`) creates and caches the frame geometry. When `draw_geometry(geom)` is called, the geometry is rendered at the renderer's current coordinate origin.

The problem is deeper than just coordinates. The interaction between:
1. iced_term's geometry cache
2. The scrollable's `with_layer` (clipping) + `with_translation` (offset)
3. The `draw_geometry` call
4. The `handle_resize` flow (called from `update()`, not `draw()`)

...creates a rendering pipeline where the terminal's cached geometry, its resize state, and the scrollable's coordinate transformations don't align.

**Specific issue with with_translation fix:**
Even following iced's Canvas pattern exactly (local coords + with_translation), the terminal still didn't render. This suggests the issue is NOT just coordinate mapping -- there may be a cache invalidation problem, a resize timing issue, or an interaction between the geometry cache and the scrollable's layer/translation stack.

### Attempt 5: Debug logging revealed Cratty-side bugs

Added file-based debug logging to both iced_term (draw/resize) and strip_view.

**Finding 1: viewport width feedback loop.** `last_size` comes from the terminal backend's resize event (the terminal's layout size). Using it to calculate viewport width creates a death spiral:
```
terminal at 1000px -> vw = 1000 - 200(sidebar) = 800
-> terminal resizes to 800 -> vw = 800 - 200 = 600
-> terminal resizes to 600 -> vw = 600 - 200 = 400
-> ... -> vw = 0, terminal = 0, gray screen
```
Fix: viewport width must come from window size, not terminal size.

**Finding 2: Second pane never created.** Strip always showed "1 panes". Ctrl+Shift+N was received by the terminal (writing ^N to the PTY) but the NewPane message either didn't fire or the pane creation failed silently.

## What We Don't Know Yet

1. **Is handle_resize called inside a scrollable?** The resize happens in `update()` which receives layout bounds. If the scrollable doesn't forward events to off-screen children, `handle_resize` never fires and the terminal backend stays at 0x0.

2. **Does the geometry cache interact poorly with with_translation?** The cache key might depend on the viewport size. If the scrollable changes the viewport between frames, the cache might be invalidating and recreating geometry incorrectly.

3. **Is the terminal backend grid actually populated?** Even with correct layout sizes, if the backend never receives a resize command, `renderable_content().grid.display_iter()` yields nothing.

4. **Does draw_geometry work inside with_layer + with_translation stacks?** The scrollable applies both. Maybe the geometry rendering doesn't compose correctly with nested transformations.

## Current Workaround

FillPortion layout without scrollable. Show focused pane + up to one adjacent pane on each side. Navigate with Ctrl+Shift+Left/Right. This works because:
- No scrollable involved
- iced's row distributes FillPortion to concrete pixel widths
- TerminalView gets real bounds via normal layout (no compression)
- Original draw() code works (absolute coordinates match frame)

## What Needs to Be Fixed in iced_term

To properly support scrollable containers, iced_term needs:

1. **layout()**: Use meaningful intrinsic size instead of `Size::ZERO` (80x24 cells)
2. **draw()**: Follow iced's Canvas widget pattern:
   - Frame sized to `layout.bounds().size()` (not `viewport.size()`)
   - Cells drawn at local coordinates (0,0 based)
   - `with_translation(bounds.x, bounds.y)` wrapping `draw_geometry`
3. **Investigate**: Why the Canvas pattern doesn't work for TerminalView specifically:
   - Cache invalidation behavior
   - handle_resize timing with scrollable event forwarding
   - Backend grid state when resize events are delayed

## Files Investigated

- `D:/Jagrit/Contributions/iced_term/src/view.rs` -- Widget impl (layout, draw, update)
- `D:/Jagrit/Contributions/iced_term/src/font.rs` -- Font measurement (cell dimensions)
- `/c/Users/jagri/.cargo/registry/src/.../iced_widget-0.14.2/src/scrollable.rs` -- Scrollable draw with_layer + with_translation
- `/c/Users/jagri/.cargo/registry/src/.../iced_widget-0.14.2/src/canvas.rs` -- Canvas widget (reference pattern for geometry rendering)
- `/c/Users/jagri/.cargo/registry/src/.../iced_core-0.14.0/src/layout/limits.rs` -- Limits resolve with compression
- `/c/Users/jagri/.cargo/registry/src/.../iced_core-0.14.0/src/renderer.rs` -- with_translation, with_layer
- `/c/Users/jagri/.cargo/registry/src/.../iced_core-0.14.0/src/widget/operation/scrollable.rs` -- Scrollable operations
