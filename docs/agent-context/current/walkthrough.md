# Phase 10: DSL Refinement (Walkthrough)

## Goal

Improve the ergonomics of the `define_language!` macro, specifically for defining `binding_pass` and `reference_pass`.

## Changes

### 1. Refactored `define_language!` Macro

- **Problem**: The `define_language!` macro used simple pattern matching which made it difficult to support optional arguments with different syntaxes (e.g., `simple("let")` vs `MyPass::new()`).
- **Solution**: Refactored the macro to use a "TT Muncher" (Token Tree Muncher) pattern. This allows the macro to recursively parse the input token stream, handling options in any order and supporting flexible syntax.
- **Implementation**:
  - Added `@parse_options` rules to `src/macros.rs`.
  - Implemented specific rules for `binding_pass = simple("kw")` and `reference_pass = simple`.
  - Maintained backward compatibility for arbitrary expressions.

### 2. Updated Examples

- Updated `examples/repl.rs` to use the new convenience syntax:
  ```rust
  binding_pass = simple("let");
  reference_pass = simple;
  ```
- Updated `examples/scoping_demo.rs` similarly.

## Key Decisions

- **TT Muncher for Options**: We chose to use the TT Muncher pattern for parsing macro options. This is a standard technique in Rust macros for handling complex, unordered, or optional arguments. It provides greater flexibility than fixed pattern matching and allows us to introduce "syntactic sugar" like `simple(...)` without breaking existing code.

