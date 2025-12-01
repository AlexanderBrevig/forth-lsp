# forth-lsp

[![CI](https://github.com/AlexanderBrevig/forth-lsp/actions/workflows/ci.yml/badge.svg)](https://github.com/AlexanderBrevig/forth-lsp/actions/workflows/ci.yml)

`forth-lsp` is an implementation of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) for the [Forth](https://forth-standard.org/) programming language.

I like forth, and I love [helix](https://github.com/helix-editor/helix)!
This project is a companion to [tree-sitter-forth](https://github.com/AlexanderBrevig/tree-sitter-forth) in order to make forth barable on helix :)

Currently this simple LSP supports `Hover`, `Completion` and `GotoDefinition`.

[Issues](https://github.com/AlexanderBrevig/forth-lsp/issues) and [PRs](https://github.com/AlexanderBrevig/forth-lsp/pulls) are very welcome!

## Install

```shell
cargo install forth-lsp
```

You can now configure your editor to use this LSP.

## Development

This is a Cargo workspace containing:

- `forth-lsp` - The main LSP server
- `lib/forth-lexer` - The Forth lexer/tokenizer library

### Testing

```shell
# Test all workspace members (both forth-lsp and forth-lexer)
cargo test --workspace

# Or use the convenient alias
cargo t

# Test only the main forth-lsp package
cargo test
```
