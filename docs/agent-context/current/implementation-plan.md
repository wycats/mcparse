# Implementation Plan: Phase 4 - Documentation & Guides

## Goal

Create comprehensive documentation for McParse to make it accessible and easy to use. This involves setting up a "Book" with tutorials and guides, and ensuring the API documentation is polished.

## Detailed Steps

### 1. Setup MdBook

- **Task**: Initialize the documentation site.
- **Details**:
  - Install `mdbook` (if not present, or just use the binary).
  - Initialize `docs/book` structure.
  - Configure `book.toml`.
  - Set up a GitHub Action (or just a script) to build/deploy (optional for now).

### 2. "The McParse Book" Content

- **Introduction & Philosophy**:
  - Explain the "Shape Algebra" and "Atomic Lexing" concepts.
  - Explain why this approach is better than Regex or Parser Combinators for IDEs.
- **Tutorial: JSON Parser**:
  - Walk through building a JSON parser from scratch.
  - Cover: Defining Atoms, Shapes, and handling delimiters.
- **Tutorial: Scripting Language**:
  - Walk through building "MiniScript" (or similar).
  - Cover: Macros, Variable Binding, Hygiene, and Expressions.
- **Advanced Topics**:
  - Deep dive into Error Recovery.
  - Deep dive into Incremental Parsing (conceptual).
  - Deep dive into Custom Shapes.

### 3. API Documentation

- **Task**: Polish Rustdoc comments.
- **Details**:
  - Ensure all public items in `src/lib.rs`, `src/shape.rs`, `src/atom.rs`, etc., have doc comments.
  - Add examples to doc comments where appropriate (doctests).
  - Run `cargo doc --open` to verify the output.
