# Reference Log

Running log of external documentation and reference material used while building Cratty features.

## 2026-04-15

### LSP installer references

- Rust Analyzer installation guide
  - https://rust-analyzer.github.io/book/installation.html
  - Used for the initial `rust-analyzer` install flow and binary naming assumptions.

- TypeScript Language Server repository
  - https://github.com/typescript-language-server/typescript-language-server
  - Used for the `typescript-language-server --stdio` command mapping.

- VS Code extracted language servers
  - https://github.com/hrsh7th/vscode-langservers-extracted
  - Used for JSON language-server install assumptions.

- YAML Language Server repository
  - https://github.com/redhat-developer/yaml-language-server
  - Used for YAML language-server install assumptions.

- Bash Language Server repository
  - https://github.com/bash-lsp/bash-language-server
  - Used for shell language-server install assumptions.

### TOML / Taplo references

- Taplo binary installation docs
  - https://taplo.tamasfe.dev/cli/installation/binary.html
  - Used to switch TOML LSP setup from `cargo install` to downloading the prebuilt Windows binary.

- Taplo cargo installation docs
  - https://taplo.tamasfe.dev/cli/installation/cargo.html
  - Used to validate the previous cargo-based install path and why it was slower.

### Syntax highlighting references

- `iced` text editor highlighter API
  - https://docs.rs/iced/latest/iced/widget/text_editor/struct.TextEditor.html
  - Used to confirm `text_editor.highlight(...)` exists and that syntax highlighting could be integrated without a custom editor renderer.

## 2026-04-16

### UI direction references

- Zed product site
  - https://zed.dev/
  - Used as the reference for editor restraint, minimal chrome, and layout hierarchy.

- Zed configuration docs
  - https://zed.dev/docs/configuring-zed
  - Used as a secondary reference for the overall Zed product/UI language around clean, low-noise editor surfaces.

- Aceternity UI
  - https://ui.aceternity.com/
  - Used as the reference for spacing rhythm, panel composition, and surface polish without copying flashy effects.

- Aceternity UI components
  - https://ui.aceternity.com/components
  - Used to study popup, card, and surface spacing patterns for Cratty’s dialog/menu system.

## Logging rule

When we rely on external docs, repos, API references, or release pages to make a product or implementation decision, append:

- date
- source link
- what decision it informed
