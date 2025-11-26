# Phase 5: API Ergonomics (Walkthrough)

## Goal
Refine the API to be "magical" for common cases (regex-based atoms) while remaining "composable" for complex cases (custom atoms).

## Changes

### 1. Declarative Atoms
We introduced `RegexAtom` and `KeywordAtom` in `src/atoms.rs`. These allow defining atoms using standard regular expressions and keyword lists, removing the need for manual state machine implementation for simple tokens.

### 2. Macro Syntax Update
We updated the `define_language!` macro to support a new, declarative syntax:

```rust
define_language! {
    struct MyLang;
    atoms = [
        atom Identifier = r"[a-zA-Z_]\w*",
        atom Number = r"\d+",
        keyword "if",
        keywords [ "else", "while" ],
    ];
    // ...
}
```

This syntax expands to `RegexAtom` and `KeywordAtom` instantiations. We also made `variable_rules` optional, defaulting to `NoOpVariableRules`.

### 3. Documentation
- Added a new chapter **"Custom Atoms"** (`docs/book/src/advanced/custom-atoms.md`) to explain the "Escape Hatch" – how to implement `Atom` manually when regex isn't enough.
- Updated the **"JSON Tutorial"** (`docs/book/src/tutorial/json/atoms.md`) to use the new declarative syntax, significantly simplifying the code.

### 4. Examples
- Updated `examples/json_plus.rs` to use the new syntax, reducing boilerplate by ~60 lines.

### 5. Further Ergonomics
- **Literal Atoms**: Added `LiteralAtom` and updated `define_language!` to support `atom Name = "literal"` syntax.
- **Regex Atoms**: Updated `define_language!` to require `regex` keyword for regex atoms: `atom Name = regex r"..."`.
- **Delimiters**: Simplified delimiter syntax: `delimiter "name" = "{", "}"`.
- **Quick Start**: Updated the Quick Start guide to use the new, concise syntax.
- **Fixes**: Restored fallback support in `define_language!` to ensure custom atoms and delimiters still work (fixing `miniscript_errors.rs` and `repl.rs`).

## Contextual Keywords
- Added a new advanced chapter (`docs/book/src/advanced/contextual-keywords.md`) explaining how to add keywords backwards-compatibly by using custom `VariableRules` and avoiding the `keywords` list. This addresses the interaction between atomic lexing (where keywords are defined) and macro expansion (where new syntax is introduced).
- **Refinement**: Clarified in documentation that variable binding is a static, lexing-time property and cannot be influenced by macro expansion.

## Verification
- **Unit Tests**: Added `src/atoms_tests.rs` and `src/macro_tests.rs` covering the new functionality.
- **Doctests**: Verified all code examples in the book.
- **Example**: Verified `examples/json_plus.rs` runs correctly.

