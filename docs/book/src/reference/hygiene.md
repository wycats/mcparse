# Hygiene & Scoping

McParse handles hygiene by classifying identifiers during the atomic lexing phase.

## `VariableRole`

Identifiers carry a `VariableRole`:

- `Binding`: A declaration of a new variable (e.g., `let x`).
- `Reference`: A usage of an existing variable (e.g., `x + 1`).
- `None`: Neither (or unknown).

## `VariableRules` Trait

The `VariableRules` trait allows the language to define how to classify identifiers based on local context.

```rust
# use std::fmt::Debug;
# use mcparse::{Token, atom::VariableRole};
pub trait VariableRules: Debug {
    fn classify(&self, prev: Option<&Token>, curr: &Token) -> VariableRole;
}
```

- `prev`: The token immediately preceding the current identifier.
- `curr`: The current identifier token.

This simple lookbehind is often enough to distinguish bindings from references in many languages (e.g., after `let`, `fn`, `class`).
