## Terminal-First Editor Design

Cratty should stay a terminal-first application. Instead of embedding a GUI text editor, file opens should launch a TUI editor inside a terminal-backed pane. This keeps fonts, cursor behavior, scrolling, and pane feel consistent with the rest of the app.

### Core model

- A workspace owns at most one replaceable preview pane.
- Normal file open actions target that preview pane.
- The preview pane is terminal-backed and launches the configured editor in read-only mode when supported.
- Pressing `i` in the focused preview pane promotes it in place to a pinned editor pane.
- A pinned editor pane is no longer eligible for replacement by future file opens.

### Interaction rules

- `Alt+E` opens the file picker.
- Choosing a file opens or reuses the workspace preview pane and scrolls to it.
- `i` commits the focused preview pane into a pinned editor session.
- `Alt+W` closes the focused pane.
- Closing the preview pane clears the workspace preview slot.

### Editor launcher

- Editor launch is configurable in app config.
- The default command is `hx`.
- The editor launches in the file's parent directory and receives the file path as an argument.
- Preview mode adds a read-only flag for known editors like Helix and Vim/Neovim.

### Implementation notes

- GUI editor code should be phased out in favor of terminal-backed editor panes.
- Pane metadata distinguishes shell panes from editor panes and preview from pinned editor sessions.
- Workspace state tracks the preview pane id so preview reuse works even when the pane is not currently focused.
