# Cratty Architecture

## What Is Cratty?

A native terminal emulator that replicates Tabby's UX without Electron. Tabby is ~200MB and runs on Chromium + Node.js. Cratty is ~2.4MB and talks directly to the GPU.

## Stack

```
┌─ cratty.exe ──────────────────────────────────┐
│                                                │
│  ┌─ iced 0.14 (Elm Architecture) ───────────┐  │
│  │                                           │  │
│  │  Custom Chrome (decorations: false)       │  │
│  │  ┌─────────────────────────────────────┐  │  │
│  │  │ Tabs  [⋮ ✕]  [+]        [─] [□] [✕]│  │  │
│  │  └─────────────────────────────────────┘  │  │
│  │                                           │  │
│  │  ┌─ iced_term 0.8 ─────────────────────┐  │  │
│  │  │  ┌─ alacritty_terminal 0.25 ──────┐ │  │  │
│  │  │  │  VT100/220/xterm parser        │ │  │  │
│  │  │  │  Terminal grid + scrollback     │ │  │  │
│  │  │  └────────────────────────────────┘ │  │  │
│  │  │  PTY backend (conpty / unix pty)    │  │  │
│  │  └─────────────────────────────────────┘  │  │
│  │                                           │  │
│  │  wgpu renderer (GPU-accelerated)          │  │
│  └───────────────────────────────────────────┘  │
│                                                │
│  Shell: pwsh.exe / bash / cmd.exe              │
└────────────────────────────────────────────────┘
```

## Workspace Crates

### cratty_app (the binary)
All GUI code lives here. Single file: `src/main.rs`.

Responsibilities:
- iced application bootstrap
- Custom titlebar with Phosphor Bold icons
- Tab management (create, close, switch, rename)
- Three-dot dropdown menu (stack overlay)
- Window chrome (drag, minimize, maximize, close)
- Terminal widget hosting via iced_term

### cratty_core (library, not yet wired)
Scaffolded types for future use:
- `AppConfig` — YAML config with serde (font, shell profiles, quake mode)
- `Session` trait — unified interface for local/SSH/serial backends
- `KeybindingSet` — configurable keybindings
- `Theme` — ANSI color palette

### cratty_vt (Zig VT parser, dormant)
A Zig-based VT parser and terminal grid. Built but unused — iced_term uses alacritty_terminal internally. Kept for the planned Zig rewrite.

## Data Flow

```
PTY (shell process)
  │
  │ bytes (stdout)
  ▼
iced_term backend (alacritty_terminal parses VT sequences)
  │
  │ Event::BackendCall(term_id, command)
  ▼
Cratty::subscription() → Message::TermEvent
  │
  │ 
  ▼
Cratty::update() → tab.term.handle(Command::ProxyToBackend(cmd))
  │
  │ returns Action::Shutdown | ChangeTitle | Ignore
  ▼
Cratty::view() → TerminalView::show(&tab.term) renders the grid
  │
  ▼
wgpu → GPU → pixels on screen
```

## Key Design Decisions

### Why custom chrome?
Tabby puts tabs in the titlebar. OS decorations don't allow this. `decorations(false)` gives us full control over the titlebar area, letting tabs sit directly in it.

### Why alacritty_terminal over custom VT parser?
We built a Zig parser but it couldn't handle PowerShell's complex escape sequences (bracketed paste, OSC title updates, prompt marks). alacritty_terminal is battle-tested and handles everything.

### Why stack overlay for menus?
iced has no built-in popover/dropdown. Rendering a menu inside a fixed-height titlebar clips it. The `stack` widget layers elements on the z-axis, so we render the menu on top of everything with an invisible scrim behind it for click-to-close.

### Why store last_size for new tabs?
iced_term's `TerminalView::handle_resize` only fires during `update()`, which requires input events. New tabs are created but never receive events until focused, so they default to a tiny size (~2 lines). Fix: capture the layout size from any tab's resize event and manually inject it into new terminals at creation time.

### Connected tab styling
Active tabs have `Radius::new(4.0).bottom(0.0)` (top rounded, bottom flat) and their background matches the terminal (`#181818`). The tab row has zero bottom padding. This makes the active tab visually merge into the terminal area — the Tabby look.

## Comparison

|                  | Tabby          | Cratty            |
|------------------|----------------|-------------------|
| Binary size      | ~200MB         | ~2.4MB            |
| Runtime          | Electron       | Native (wgpu)     |
| Language         | TypeScript     | Rust              |
| Memory           | ~300MB+        | ~30MB             |
| Terminal engine  | xterm.js       | alacritty_terminal|
| GUI framework    | Chromium       | iced 0.14         |
| Architecture     | Component-based| Elm (functional)  |

## Known Limitations

### iced_term doesn't expose child PID
The PTY child process PID is created internally in `Backend::new` and never surfaced. This blocks:
- **Duplicate foreground process**: Can't detect what's running (e.g., vim, Claude Code) to re-launch it in the duplicated tab. Currently we only inherit the CWD by parsing the shell-reported title.
- **Process-aware tab titles**: Can't show the foreground command name (like Tabby/iTerm2 do).
- **Graceful close**: Can't send SIGHUP/SIGTERM to the child before closing.

Workarounds considered:
1. Fork iced_term to expose `tty::Pty` or at least the child PID
2. On Windows: enumerate child processes of our own PID via `CreateToolhelp32Snapshot`, match by creation time against tab creation order — fragile
3. On Linux: walk `/proc/<our_pid>/task/*/children` — simpler but platform-specific
4. Build our own PTY layer (planned for Zig rewrite) with full process tree access

### No OSC 7 (CWD reporting) support
Modern shells can emit OSC 7 to report the current working directory. alacritty_terminal parses it but iced_term doesn't surface it as an event. This means CWD detection relies on title parsing, which is fragile across shell configurations.

## Future Plans

1. Wire cratty_core config into the app
2. Split panes (horizontal/vertical)
3. SSH client integration
4. Settings panel
5. Quake-mode dropdown terminal
6. Full Zig rewrite using Ghostty's approach
