# forth-lsp

[![CI](https://github.com/AlexanderBrevig/forth-lsp/actions/workflows/ci.yml/badge.svg)](https://github.com/AlexanderBrevig/forth-lsp/actions/workflows/ci.yml)

A [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) implementation for [Forth](https://forth-standard.org/), bringing modern IDE features to Forth development.

## Features

- **Hover** - View documentation for built-in words and user-defined functions
- **Completion** - Auto-complete for built-in words and your definitions
- **Go to Definition** - Jump to where words are defined
- **Find References** - Find all usages of a word
- **Rename** - Rename symbols across your workspace
- **Document Symbols** - Outline view of word definitions in current file
- **Workspace Symbols** - Search for definitions across all files
- **Signature Help** - View parameter information while typing
- **Diagnostics** - Real-time error detection for undefined words
- **Formatting** - Auto-format your code with configurable options

## Installation

```shell
cargo install forth-lsp
```

Then configure your editor to use `forth-lsp`. Works with any LSP-compatible editor (VS Code, Neovim, Helix, Emacs, etc.).

## Configuration

Create a `.forth-lsp.toml` in your workspace root:

```toml
[builtin]
# Point at word list files (e.g. gforth -e 'words bye' > gforth.words)
word_files = ["gforth.words"]

# Or define words inline
[[builtin.words]]
word = "MY-WORD"
stack = "( n -- )"
description = "Does something special"

[format]
indent_width = 2
use_spaces = true
```

## Contributing

[Issues](https://github.com/AlexanderBrevig/forth-lsp/issues) and [PRs](https://github.com/AlexanderBrevig/forth-lsp/pulls) welcome!

### Development

```shell
# Run tests
cargo test --workspace
# or
cargo t
```
