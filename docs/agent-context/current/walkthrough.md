# Phase 6: Variable Binding & Scoping Architecture (Walkthrough)

## Goal

Implement a robust, 4-step parsing architecture that decouples variable binding from the initial lexical analysis, enabling proper scoping rules while maintaining a simple atomic lexer.

## Changes

### 1. Architectural Shift

We moved from a single-pass "Lexer + VariableRules" model to a multi-pass model:

1.  **Lexing**: Produces `TokenTree`.
2.  **Binding Pass**: Identifies declarations (`BindingPass` trait).
3.  **Reference Pass**: Resolves references (`ReferencePass` trait).
4.  **Parsing**: Matches shapes.

### 2. Core Data Structures

- **`AtomKind`**: Removed `VariableRole` from `AtomKind::Identifier`. Identifiers are now just identifiers during lexing.
- **`Token`**: Added a `binding: Option<BindingId>` field to store the resolution result.
- **`Language` Trait**: Replaced `variable_rules()` with `binding_pass()` and `reference_pass()`.

### 3. Scoping Infrastructure (`src/scoping.rs`)

- Defined `ScopeStack` to manage nested scopes.
- Defined `BindingPass` and `ReferencePass` traits.
- Provided `NoOpBindingPass` and `NoOpReferencePass` as defaults.
- **Implemented `SimpleBindingPass`**: A default implementation that scans for a keyword (e.g., `let`) followed by an identifier to define bindings. It handles block recursion automatically.
- **Implemented `SimpleReferencePass`**: A default implementation that resolves identifiers against the `ScopeStack` populated by the binding pass.
- **Implemented `scope_tokens`**: A helper function to orchestrate the passes.

### 4. Verification

- Created `examples/scoping_demo.rs` to verify the scoping logic.
- Confirmed that:
    - Variable definitions are correctly identified.
    - References are resolved to the correct `BindingId`.
    - Shadowing works as expected (inner scopes override outer scopes).
    - References before definition (hoisting check) remain unbound.

### 5. Documentation

- Updated **The McParse Book** to describe the new architecture in "Hygiene & Scoping", "Contextual Keywords", and "Quick Start".
- Updated `docs/design/variable-binding-architecture.md` with a retrospective on why the previous approach failed.

### 6. Migration

- Updated `miniscript` and `repl` examples to compile with the new API.
- Fixed widespread compilation errors caused by the breaking changes to `Token` and `AtomKind`.
