# iced 0.14 Patterns & Undocumented APIs

Reference for working with iced 0.14 — most of this was discovered by reading source code since docs are minimal.

## Application Bootstrap

iced 0.14 uses a function-based API, not the old `Application` trait:

```rust
iced::application(App::new, App::update, App::view)
    .decorations(false)          // remove OS titlebar
    .window_size((1200.0, 800.0))
    .font(FONT_BYTES)            // embed font at compile time
    .theme(App::theme)           // NOT a closure — lifetime trap
    .run()
```

**Gotcha:** `.theme(|_| Theme::Dark)` fails with "implementation of FnOnce is not general enough". Use a method reference: `.theme(App::theme)`.

## The Elm Architecture

```
State → view() → Element tree → iced diffs & renders
  ↑                                    │
  │                              user interaction
  │                                    │
  └── update(msg) ◄── Message ◄────────┘
       returns Task (async side effects)
```

- `view(&self)` takes an immutable reference — you cannot mutate state during rendering
- `update(&mut self, msg)` is the only place state changes
- `Task<Message>` handles async work (focus widget, close window, etc.)
- The entire widget tree is rebuilt every frame; iced diffs internally

## Window Management (Custom Chrome)

`window::Id` has no public constructor or `MAIN` constant in iced 0.14. Use `window::oldest()`:

```rust
fn with_window<F, T>(f: F) -> Task<T>
where
    F: Fn(window::Id) -> Task<T> + Send + 'static,
    T: Send + 'static,
{
    window::oldest().and_then(move |id| f(id))
}
```

All window operations:

```rust
Message::DragWindow  => with_window(window::drag),
Message::Minimize    => with_window(|id| window::minimize(id, true)),
Message::Maximize    => with_window(window::toggle_maximize),
Message::CloseWindow => with_window(window::close),
```

**Do not use `std::process::exit(0)`** — it doesn't let iced clean up GPU resources. Always use `window::close`.

## Titlebar Drag

Wrap the titlebar in `mouse_area` with `on_press(Message::DragWindow)`:

```rust
mouse_area(
    container(bar_inner).width(Length::Fill)
)
.on_press(Message::DragWindow)
```

## Phosphor Bold Icons

Embed the TTF at compile time, reference by font name and Unicode codepoint:

```rust
const PHOSPHOR_BOLD_BYTES: &[u8] = include_bytes!("../resources/fonts/Phosphor-Bold.ttf");
const PHOSPHOR: Font = Font::with_name("Phosphor-Bold");

// Codepoints (Phosphor Bold, from CSS/web font metadata):
// U+E32A  minus (minimize)
// U+E45E  square (maximize)
// U+E4F6  x (close)
// U+E3D4  plus (new tab)
// U+E208  dots-three-vertical (kebab menu)
```

Register the font: `.font(PHOSPHOR_BOLD_BYTES)` on the application builder.

Usage: `text(ICO_X).font(PHOSPHOR).size(14)`

### Centering Icons in Buttons

**Do NOT use `text.width(Fill).height(Fill).center()`** inside buttons. It does not reliably center Phosphor glyphs. Instead, wrap in a `container` with `.center(Length::Fill)`:

```rust
fn centered_icon(codepoint: char, size: f32) -> Element<'static, Message> {
    container(
        text(codepoint)
            .font(PHOSPHOR)
            .size(size)
            .shaping(text::Shaping::Advanced),
    )
    .center(Length::Fill)
    .into()
}

// Button with fixed size, padding(0), hover color via text_color:
button(centered_icon(ICO_X, 12.0))
    .width(18)
    .height(18)
    .padding(0)
    .style(move |_, status| button::Style {
        text_color: match status {
            button::Status::Hovered => FG_ACTIVE,
            _ => fg,
        },
        ..Default::default()
    })
```

Key rules:
- Do NOT set `.color()` on the text -- it prevents button style `text_color` from working (needed for hover)
- Always use fixed `width` + `height` on the button -- `Length::Fill` causes expansion into empty space
- `padding(0)` -- the container handles centering, padding would offset it

## Per-Corner Border Radius

`iced::border::Radius` supports per-corner values via builder methods:

```rust
// Top corners rounded, bottom flat (for tabs connecting to content)
iced::border::Radius::new(4.0).bottom(0.0)

// Other methods: .top(), .left(), .right(), .top_left(), .bottom_right(), etc.
```

`[f32; 4]` does NOT implement `Into<Radius>`. Use the builder or `f32.into()` for uniform.

## Padding

`iced::Padding` only accepts `f32`, `u16`, `[f32; 2]`, or `[u16; 2]` via `Into`. For 4-sided padding use the struct directly:

```rust
iced::Padding { top: 4.0, right: 8.0, bottom: 0.0, left: 8.0 }
```

## Overlay / Dropdown Menus

iced has no built-in popover. Use `stack` to layer elements on top of each other:

```rust
use iced::widget::stack;

// Layer 1: main content
// Layer 2: invisible click-to-close scrim
// Layer 3: menu positioned with padding
stack![main_content, scrim, menu_overlay]
    .width(Length::Fill)
    .height(Length::Fill)
```

Position the menu using container padding from the top-left corner.

## Focus Management

Focus the next focusable widget (useful after showing a text input):

```rust
iced::widget::operation::focus_next()
```

`iced::widget::focus_next()` does NOT exist at that path — it's under `operation`.

## Keyboard Events

`keyboard::on_key_press` does not exist in iced 0.14. Use `event::listen_with`:

```rust
event::listen_with(|evt, status, _window| {
    if let iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = evt {
        Some(Message::KeyPressed(key, modifiers))
    } else {
        None
    }
})
```

Note: `keyboard::listen()` only captures **ignored** events (not consumed by widgets). If you need to catch Escape while a text_input is focused, use `event::listen_with` which sees all events regardless of status.

### Character Key Matching with Modifiers

iced reports `keyboard::Key::Character` as **lowercase** even when Shift is held. Always use case-insensitive comparison:

```rust
// WRONG: will never match because iced sends "t" not "T"
if c.as_str() == "T" { ... }

// CORRECT:
if c.as_str().eq_ignore_ascii_case("t") { ... }
```

## Button text_color vs Text .color()

`button::Style.text_color` only affects child `Text` widgets that do **NOT** have an explicit `.color()` set. If you call `.color(fg)` on the text, the button style cannot override it.

This matters for hover effects. To change icon color on hover:

```rust
// WRONG: .color(fg) bakes in the color, hover text_color is ignored
button(text("X").color(fg))
    .style(|_, status| button::Style {
        text_color: match status {
            Hovered => WHITE,  // ignored because .color() takes precedence
            _ => fg,
        },
        ..Default::default()
    })

// CORRECT: omit .color(), let text_color control it
button(text("X"))  // no .color() call
    .style(|_, status| button::Style {
        text_color: match status {
            Hovered => WHITE,  // works
            _ => fg,
        },
        ..Default::default()
    })
```

## Text Input Styling

```rust
text_input("placeholder", &value)
    .on_input(Message::Input)
    .on_submit(Message::Submit)
    .size(11)
    .style(|_, _status| text_input::Style {
        background: iced::Background::Color(bg),
        border: iced::Border { color, width: 1.0, radius: 3.0.into() },
        icon: icon_color,
        placeholder: placeholder_color,
        value: value_color,
        selection: selection_color,
    })
```

Text input consumes Escape internally (unfocuses itself). There is no `on_escape` callback.

## iced::time::every requires tokio feature

`iced::time::every()` is behind the `tokio` feature flag. Without it, the module exists but `every` is missing:

```toml
iced = { version = "0.14", features = ["canvas", "advanced", "lazy", "tokio"] }
```

## Canvas Geometry Coordinate System

Canvas geometry uses LOCAL coordinates (0,0 based). iced positions the geometry at the widget's layout bounds via `with_translation`. This is critical for scrollable support.

```rust
// WRONG: absolute screen coordinates in the frame
let x = layout.position().x + col * cell_w;  // breaks in scrollable

// CORRECT: local coordinates, let iced handle positioning
let x = col * cell_w;  // starts from 0,0
```

Frame size should be `layout.bounds().size()`, NOT `viewport.size()`. The viewport is the visible window, but the frame represents the widget's own dimensions.

Reference: iced's own Canvas widget (`iced_widget/src/canvas.rs` lines 300-310) uses this pattern.

## Scrollable Layout Compression

iced's scrollable uses a "compression" system for the scroll axis:
- Sets max to `f32::INFINITY` for the scroll direction
- Sets `compression = true` for Fill children
- `limits.resolve(Fill)` with compression returns `intrinsic_size`, not max

This is CORRECT behavior. The scrollable needs to know content size. The fix for Fill widgets that render blank is to provide a meaningful intrinsic size, not `Size::ZERO`.

## Terminal Resize Feedback Loop

Never use terminal layout size (`last_size` from resize events) to calculate viewport width. This creates a death spiral:

```
terminal at 1000px -> vw = 1000 - sidebar = 800
-> terminal resizes to 800 -> vw = 800 - sidebar = 600
-> ... -> vw = 0, gray screen
```

Use the window size or container bounds instead.
