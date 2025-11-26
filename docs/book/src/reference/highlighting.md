# Syntax Highlighting

McParse generates semantic tokens for syntax highlighting.

## The `Highlighter` Trait

The `Highlighter` trait is used by `Atom::highlight` to emit tokens.

```rust
# use mcparse::{Token, HighlightStyle};
pub trait Highlighter {
    fn highlight(&mut self, token: &Token, style: HighlightStyle);
}
```

## `HighlightStyle`

The `HighlightStyle` enum defines the semantic category of the token.

- `None`
- `Keyword`
- `String`
- `Number`
- `Operator`
- `Comment`
- `Variable` (Generic)
- `VariableBinding` (Declaration)
- `VariableReference` (Usage)
- `Function`
- `Type`

By using `VariableBinding` and `VariableReference`, you can achieve "semantic highlighting" (e.g., coloring declarations differently from usages) purely from the lexer output.
