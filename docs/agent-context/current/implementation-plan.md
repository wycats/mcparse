# Phase 9: Scoping & Completion (Implementation Plan)

## Goal

Enhance the developer experience by implementing intelligent Tab Completion for variables in the REPL. This requires refining the `VariableRules` and `BindingPass` architecture to be fully robust and context-aware.

## Detailed Steps

### 1. Refactor `VariableRules`

- Decouple `VariableRules` from the raw lexer.
- Ensure it operates purely as a post-lexing pass on `TokenTree`.
- Give it access to the delimiter stack (context) to handle complex binding patterns (e.g., `let (a, b)`).

### 2. Implement Tab Completion

- Add a `complete` method to the `Language` trait (or a new `Completion` trait).
- Implement logic to:
  1.  Parse the current input (partial parse).
  2.  Identify the cursor position in the `TokenTree`.
  3.  Traverse the `ScopeStack` at that position.
  4.  Return a list of available bindings.
- Integrate this into the `repl` example.

### 3. Verify Scoping Logic

- Add tests for block scoping, shadowing, and hoisting.
- Ensure the `BindingPass` correctly handles unclosed delimiters (common during completion).
