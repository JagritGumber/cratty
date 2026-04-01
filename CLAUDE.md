# Cratty

A native terminal emulator written in Rust + Zig, inspired by Tabby's UX.

## Architecture

- **cratty_core**: Core traits, config, event types (no UI deps)
- **cratty_vt**: Zig-based VT parser + terminal grid, exposed to Rust via C ABI FFI
- **cratty_terminal**: PTY spawning + session management, wraps cratty_vt
- **cratty_ssh**: SSH2 client via russh
- **cratty_serial**: Serial port terminal via serialport
- **cratty_ui**: GUI layer (GPUI planned, TUI bootstrap for now)
- **cratty_plugin**: Dynamic plugin system
- **cratty_app**: Binary entry point

## Build

Requires Zig (0.14+) and Rust (1.85+) in PATH.

```
cargo build
```

The build.rs in cratty_vt automatically invokes `zig build` to compile the VT engine.

## Key design decisions

- Zig handles the hot path: VT parsing and grid management (packed 12-byte cells, arena scrollback)
- Rust handles everything else: PTY, SSH, config, UI, plugin system
- Session trait unifies local/SSH/serial backends
- Config is YAML with strongly-typed serde structs
