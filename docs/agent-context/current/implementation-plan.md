# Implementation Plan: Prototyping Core Traits

## Goal

Translate the design sketches into actual Rust code, creating the foundational traits and types for McParse. We will also build a basic highlighting printer and error reporter to visually verify our progress.

## Steps

1.  **Project Setup**: Ensure `Cargo.toml` has necessary dependencies (e.g., `tokio` for async, `thiserror` for errors, `miette` for error reporting, `owo-colors` or similar for highlighting).
2.  **Core Traits**: Implement the traits defined in `DESIGN.md` in `src/lib.rs` (or appropriate modules):
    - `Atom` and `AtomKind`
    - `Shape` and `AdjacencyConstraint`
    - `Macro` and `MacroContext`
    - `Language`
    - `Highlighter` (new trait for syntax highlighting)
3.  **Token Types**: Define `Token`, `TokenTree`, `Cursor`, and `SourceLocation`. Ensure `SourceLocation` is compatible with `miette`.
4.  **Mock Implementation**: Create a dummy implementation of `Atom` (e.g., `WhitespaceAtom`, `IdentifierAtom`) to verify the traits compile and compose.
5.  **Visual Verification**:
    - Implement a simple `ANSIHighlighter` that implements `Highlighter`.
    - Create a demo binary that:
      - Constructs a simple `Shape`.
      - Parses a string.
      - Prints the highlighted output to the terminal.
      - Intentionally triggers a parse error and prints the `miette` report to verify source locations.
