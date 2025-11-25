# Walkthrough: Prototyping Core Traits

## Summary
In this phase, we translated the design into actual Rust code. We implemented the foundational traits and types, and verified them with a working demo that includes syntax highlighting and error reporting.

## Key Implementations

### Core Traits
We defined the following traits in `src/lib.rs` and associated modules:
-   `Atom`: Defines lexical units. Includes `parse` and `highlight` methods.
-   `Shape`: Defines syntactic structures. Includes `match_shape` and `adjacency`.
-   `Macro`: Defines macro expansion logic.
-   `Language`: Defines the set of atoms, delimiters, and macros.
-   `Highlighter`: Defines how to colorize tokens.

### Token Types
We defined `Token`, `TokenTree`, `Cursor`, and `SourceLocation` in `src/token.rs`. `SourceLocation` wraps `miette::SourceSpan` for easy integration with the error reporting ecosystem.

### Visual Verification
We created a demo in `src/main.rs` that:
1.  Uses mock atoms (`WhitespaceAtom`, `IdentifierAtom`).
2.  Parses a string using a simple loop (simulating a basic lexer).
3.  Highlights the output using `ANSIHighlighter` and `owo-colors`.
4.  Demonstrates error reporting with `miette` when parsing fails.

## Artifacts
-   `src/lib.rs`: Module exports.
-   `src/atom.rs`, `src/shape.rs`, `src/macro.rs`, `src/language.rs`, `src/highlighter.rs`: Trait definitions.
-   `src/token.rs`: Token types.
-   `src/mock.rs`: Mock implementations.
-   `src/main.rs`: Demo application.

