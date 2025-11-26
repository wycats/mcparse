# The Macro System

Macros allow you to extend the language syntax dynamically.

## The `Macro` Trait

```rust
# use std::fmt::Debug;
# use mcparse::{Shape, TokenTree, MacroContext, ExpansionResult};
# use mcparse::shape::{Precedence, Associativity};
pub trait Macro: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn signature(&self) -> &dyn Shape;
    fn expand(
        &self,
        args: TokenTree,
        lhs: Option<TokenTree>,
        context: &MacroContext,
    ) -> ExpansionResult;

    // Operator support
    fn is_operator(&self) -> bool { false }
    fn precedence(&self) -> Precedence { Precedence(0) }
    fn associativity(&self) -> Associativity { Associativity::Left }
}
```

- `name`: The keyword that triggers the macro.
- `signature`: The shape of the arguments following the keyword.
- `expand`: The transformation logic.
- `is_operator`: Whether this macro acts as an infix/postfix operator.
- `precedence` / `associativity`: Configuration for operator precedence parsing.

## Expansion Result

```rust
# use mcparse::TokenTree;
pub enum ExpansionResult {
    Ok(TokenTree),
    Error(String),
}
```

The result of expansion is a `TokenTree`, which is then inserted into the parse tree.
