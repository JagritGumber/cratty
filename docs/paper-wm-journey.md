# Building a Paper Window Manager in a Rust Terminal Emulator

## What is a Paper Window Manager?

Traditional terminal emulators use tiling: split the screen into regions, each window gets a fraction. Four terminals = 25% each. Want two terminals at 80% width? Impossible.

Paper WM flips this: windows live on an infinite horizontal strip. Each window defines its own size. You scroll through them like sheets of paper on a desk. Two windows at 80%? No problem -- they just extend the strip.

This is the approach used by Niri (a Wayland compositor). We're building it inside a terminal emulator (Cratty) using Rust and iced.

## Architecture

The key insight: tiling and paper are opposing philosophies. Tiling says "divide available space." Paper says "each window owns its size." They can't coexist in the same layer.

Our hierarchy:
- Sidebar on the left: list of workspaces (like VS Code's explorer)
- Each workspace IS a paper strip: horizontal scrollable list of terminal panes
- Each pane defines its own width

Data model:
- Workspaces stored as Vec<Workspace>
- All terminal panes in a flat HashMap<PaneId, Pane>
- Paper strips hold PaneId references, not owned terminals
- This avoids deep nested mutable borrows in Rust's ownership model

## The iced_term Scrollable Problem

First major wall: iced_term (the terminal widget) doesn't render inside a horizontal scrollable with fixed pixel widths. The terminal goes completely gray/blank.

Why? iced_term's TerminalView appears to need Length::Fill to properly measure and render. Inside a scrollable, the "available width" is infinite (that's how scrollables work), so the terminal can't determine its actual size.

This is a fundamental tension: paper WM needs fixed-width panes in a scrollable container, but the terminal widget needs flexible sizing.

Current workaround: show one pane at a time, navigate with keyboard shortcuts. The data model is fully paper WM (horizontal strip, per-pane widths, focus navigation) -- just the visual rendering is constrained to single-pane view.

Potential solutions being explored:
1. Fork iced_term to support fixed-width rendering
2. Use a manual viewport (no iced scrollable) with container clipping
3. Render terminals offscreen and blit to a canvas widget

## What We Learned About iced 0.14

### Icon Centering
Phosphor font icons won't center with text.center() inside buttons. You need:
```rust
container(text(icon).font(PHOSPHOR).size(12).shaping(Advanced))
    .center(Length::Fill)
```
Container centering works. Text centering doesn't.

### Button Hover Colors
button::Style.text_color only works on Text children WITHOUT explicit .color(). If you call .color(fg) on the text, the button style can't override it for hover effects.

### Keyboard Character Matching
iced reports Key::Character as lowercase even with Shift held. Always use eq_ignore_ascii_case.

### Focus Management
iced_term::TerminalView::focus() returns a Task that must be chained from update(). Focus is lost on every widget tree rebuild unless explicitly maintained.

## Build Phases

Phase 0: Decomposed 922-line monolith into 8 modules (style, widgets, message, terminal, titlebar, menu, sidebar, strip_view)

Phase 1: Core data structures (PaperStrip, ViewOffset with ease-out animation, FocusState machine, Workspace, IdGen)

Phase 2: Replaced Vec<Tab> with Vec<Workspace> + HashMap<PaneId, Pane>. Same UX, new data model.

Phase 3: Paper strip rendering + sidebar. Hit the scrollable rendering wall with iced_term.

Phase 4: Deep investigation of iced_term rendering bugs. Found three issues: Size::ZERO intrinsic, absolute coords in local frame, viewport width feedback loop. Confirmed iced's scrollable compression is correct by design (closed iced-rs/iced#3299). FillPortion workaround working.

Phase 5: Dropped iced_term entirely. Built our own Canvas-based terminal renderer using alacritty_terminal directly. Three new files: term_backend.rs (PTY wrapper), term_canvas.rs (grid renderer), term_widget.rs (Canvas Program + keyboard). Canvas uses local coords + with_translation natively, which is the pattern iced's scrollable supports.

## Architecture

```
alacritty_terminal (PTY + VT parsing, battle-tested)
    |
term_backend.rs (spawn, write, resize, event drain)
    |
term_canvas.rs (draw grid cells at local coords on Canvas)
    |
term_widget.rs (Canvas Program + keyboard input via Notifier)
    |
strip_view.rs (FillPortion layout, focused + adjacent panes)
    |
iced (layout, window management, scrollable)
```

## What's Next

- Test Canvas terminal inside scrollable (should work -- Canvas is iced's own widget)
- True paper WM clipping (each pane keeps its width, viewport clips overflow)
- Configurable pane widths (proportion or fixed pixels)
- Smooth scroll animation between panes (spring physics, inspired by Niri)
- Fix keyboard leak (Ctrl+Shift+N writes ^N to terminal)
- Programmable terminal I/O for AI integration
- Sidebar collapse toggle
- Workspace context menus (rename, color, delete)
