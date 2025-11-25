# Changelog

## Phase 1: Design & Prototyping (Completed)

- **Design**: Solidified `DESIGN.md` with "Expression Continuation" model and "Atomic Whitespace" handling.
- **Design**: Defined core Rust traits: `Atom`, `Shape`, `Macro`, `Language`.
- **Docs**: Added `docs/design/aspirational-language.md` with a JsonPlus example.
- **Implementation**: Created `examples/demo.rs` with a working lexer, highlighter, and `miette` error reporting.
- Initialized project structure and agent context.

## Phase 2: Core Parsing Engine & Semantics (Completed)

- **Shape Algebra**: Implemented core combinators (`term`, `seq`, `choice`, `rep`, `enter`, `adjacent`, `empty`, `end`) and derived combinators (`opt`, `separated`, `joined`) in `src/shape.rs`.
- **Atomic Lexer**: Implemented recursive descent lexer in `src/lexer.rs` producing `TokenTree`s.
- **Variable Binding**: Implemented `VariableRules` trait and integrated it into the lexer to classify identifiers as Bindings or References.
- **Macro Expansion**: Implemented `Parser` struct with `parse_expression` loop handling prefix and infix macros with precedence.
- **Error Recovery**: Implemented `recover` combinator and `TokenTree::Error` for robust parsing.
- **Integration Tests**:
  - Implemented `JsonPlus` example language demonstrating recursive shapes and delimited groups.
  - Implemented `MiniScript` example language demonstrating macros and variable binding.
- **Refactor**: Converted Shape constructors to return concrete types to support `Clone`.
