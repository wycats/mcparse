# The Macro System

Macros allow you to extend the language syntax dynamically.

## The `Macro` Trait

```rust
# use std::fmt::Debug;
# use mcparse::{Shape, TokenTree, MacroContext, ExpansionResult};
pub trait Macro: Debug {
    fn name(&self) -> &str;
    fn signature(&self) -> &dyn Shape;
    fn expand(
        &self,
        args: TokenTree,
        lhs: Option<TokenTree>,
        context: &MacroContext,
    ) -> ExpansionResult;
}
```

- `name`: The keyword that triggers the macro.
- `signature`: The shape of the arguments following the keyword.
- `expand`: The transformation logic.

## Expansion Result

```rust
# use mcparse::TokenTree;
pub enum ExpansionResult {
    Ok(TokenTree),
    Error(String),
}
```

The result of expansion is a `TokenTree`, which is then inserted into the parse tree.
