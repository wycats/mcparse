# Implementation Plan: Phase 3 - Tooling & Incrementalism

## Goal

Enhance the developer experience and tooling around McParse. This includes better debugging output, reducing boilerplate with macros, and creating interactive demos to showcase highlighting and completion. We will also lay the groundwork for incremental parsing.

## Steps

1.  **Pretty Printer**: Implement a custom `Debug` or `Display` for `TokenTree` to make it easier to read (compact but informative).
2.  **Macros**: Create `macro_rules!` to simplify the definition of Atoms and Shapes, reducing the verbosity seen in `examples/json_plus.rs`.
3.  **Syntax Highlighting**: Refine the highlighter to support more semantic tokens and ensure it works well with the new macros.
4.  **Tab Completion**: Implement the `complete` method in `Shape` and the necessary infrastructure to suggest completions at a given cursor position.
5.  **Interactive Demos**:
    - Build a simple TUI (using `crossterm` or similar) to demonstrate real-time highlighting.
    - Build a demo for tab completion where the user can press Tab to see suggestions.
6.  **Incrementalism**: (Stretch) Begin exploring incremental re-lexing and parsing.
