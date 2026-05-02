# cratty

cross-platform Rust terminal + code editor with LSP, built for AI-coding workflows.

## what it is

cratty is a terminal emulator with an integrated code editor, designed for working with multiple AI coding agents (Claude Code, Cursor, Aider, and similar) on macOS, Linux, and Windows. instead of tiling panes, it uses a paper-style window manager that keeps context visible without taking over the screen.

## why it exists

most terminals on Windows are either Electron-bloated or Linux-first projects with weak Windows support. cratty was built because:

- AI coding agents need a terminal that stays out of the way
- pseudo-console handling on Windows (ConPTY) is genuinely awkward and most terminals get the edge cases wrong
- voice-input tools like Wispr Flow need a host that does not interfere with their input pipeline

cratty handles cross-platform pseudo-console (Unix PTY + Win32 ConPTY) correctly, ships LSP support for inline editing, and works cleanly with external voice tools.

## features

- cross-platform: macOS, Linux, Windows
- pseudo-console handling per OS (Unix PTY + Win32 ConPTY)
- integrated code editor with LSP support
- paper-style window manager (replaces tiling panes)
- works with Claude Code, Cursor, Aider, and other AI coding agents
- optimized to work cleanly with external voice-input tools (Wispr Flow, etc.)

## build

```bash
cargo build --release
```

binary will be in `target/release/cratty`.

## status

actively developed. LSP integration is working but not yet polished. expect rough edges around window manager edge cases.

## license

see LICENSE file.
