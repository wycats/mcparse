# Hygiene & Scoping

McParse handles hygiene by classifying identifiers during a post-lexing pass.

## `BindingPass` and `ReferencePass`

Scoping is handled by two traits on the `Language`:

- `BindingPass`: Identifies which tokens declare new variables (bindings).
- `ReferencePass`: Identifies which tokens refer to existing variables (references) and resolves them to their bindings.

This separation allows for complex scoping rules (like block scoping, shadowing, and hoisting) to be implemented cleanly.

## `BindingPass` Trait

```rust
# use std::fmt::Debug;
# use mcparse::{token::TokenTree, scoping::ScopeStack};
pub trait BindingPass: Debug + Send + Sync {
    fn identify_bindings(&self, tokens: &mut [TokenTree], scope: &mut ScopeStack);
}
```

The `BindingPass` traverses the `TokenTree` and marks tokens as bindings by setting their `binding` field. It also pushes and pops scopes on the `ScopeStack`.

## `ReferencePass` Trait

```rust
# use std::fmt::Debug;
# use mcparse::{token::TokenTree, scoping::ScopeStack};
pub trait ReferencePass: Debug + Send + Sync {
    fn resolve_references(&self, tokens: &mut [TokenTree], scope: &mut ScopeStack);
}
```

The `ReferencePass` traverses the `TokenTree` and resolves references by looking them up in the `ScopeStack`. If a reference is found, it links the token to the corresponding binding. Note that `scope` is mutable because the pass needs to register definitions as it encounters them (so subsequent references in the same scope can resolve to them).

## Execution Order

1.  **Lexing**: The raw text is converted into a `TokenTree`.
2.  **Binding Pass**: The `BindingPass` runs, identifying declarations.
3.  **Reference Pass**: The `ReferencePass` runs, resolving references.
4.  **Macro Expansion**: Macros are expanded.
5.  **Parsing**: The shape algebra matches against the fully scoped and expanded token tree.
