# Theme & UI Design Notes

Collected design direction and pain points.

## 1. Quake mode hotkey

Tabby's Ctrl+Space toggle (show/hide) is the right UX but wrong keybinding. Ctrl+Space conflicts with:
- Wispr Flow (STT dictation trigger)
- VS Code IntelliSense
- Zed autocomplete

Need a different default. Must be configurable. Candidates TBD.

## 2. Sidebar typography

- "Workspaces" header text is too small, needs to be bigger
- Individual workspace names are almost the right size
- Truncated titles like "C:\Window..." are annoying -- need smarter truncation or show the meaningful part (project name, not drive letter)

## 3. Project folder workspaces

Opening a project folder should create a workspace named after that folder. Current behavior names workspaces "Terminal N" which is meaningless. The CWD is already tracked per pane -- derive the workspace name from the project root.

## 4. Dropdown/menu theming

The context menu (rename, delete, color swatches) uses colors that don't match the app theme. Needs to be derived from the same palette as the rest of the UI.

## 5. Two-color background philosophy (Zed-inspired)

The goal: only two background colors in the entire app.

| Surface          | Color         |
|------------------|---------------|
| Sidebar/navbar   | Color A       |
| Content/editor   | Color B       |
| Dropdowns/menus  | Color A + shadow, no borders |

Zed does this well: clean, minimal, no visible borders on menus, just shadows. But Zed leans too far into pure black. Cratty should be dark but not that dark -- warmer or slightly lifted.

No borders on dropdowns. Shadows provide separation.

## 6. User-editable themes

Users should be able to create custom themes with minimal knowledge. Current approach (hardcoded color constants in style.rs) makes this impossible for end users.

Future: theme file (TOML/YAML) that defines the two background colors + accent + foreground levels, and everything derives from those few values. Low number of user-facing knobs, high impact.

## Reference: Zed as baseline

Zed One Dark is the structural foundation (two-color model, shadow-based menus, no borders). But Cratty should NOT be an exact copy. Make it distinctly Cratty.

**Direction: warmer.** Shift the blue-gray base toward a warmer tone (slight purple or teal tint). Warmer dark themes are easier on the eyes for long sessions (f.lux principle). Popular warm dark themes: Dracula (purple), Gruvbox (brown), Catppuccin (pastel). Zed's pure blue-gray is actually the outlier.

**Accent color:** TBD. Could move away from Zed's blue to something that pairs with the warmer base.

**Status:** Currently shipping exact Zed One Dark values as a starting point. Next step is tuning the warmth and picking a unique accent.
