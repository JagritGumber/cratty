# Cratty

A native terminal emulator written in Rust, inspired by Tabby's UX but without Electron bloat.

## Architecture

- **cratty_app**: Binary entry point — iced 0.14 GUI with custom window chrome, tabbed terminals (iced_term + alacritty_terminal)
- **cratty_core**: Core traits, config, event types (not yet wired into the app)
- **cratty_vt**: Zig-based VT parser + terminal grid (future use — currently iced_term handles VT internally)

## Build

Requires Rust (1.85+) in PATH.

```
cargo build --release
```

Binary output: `target/release/cratty.exe`

## Key design decisions

- Uses iced_term (wraps alacritty_terminal) for terminal rendering
- Custom window chrome with `.decorations(false)` — Phosphor Bold icons for window controls
- Tabs styled to visually connect to the terminal area (Tabby-style)
- Future plan: full Zig rewrite using Ghostty's approach
