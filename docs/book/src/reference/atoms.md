# The Atom Trait

The `Atom` trait defines a lexical unit in your language.

```rust
# use std::fmt::Debug;
# use mcparse::{AtomKind, token::{Token, Cursor}, highlighter::Highlighter};
pub trait Atom: Debug {
    fn kind(&self) -> AtomKind;
    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)>;
    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter);
}
```

## `AtomKind`

The `AtomKind` enum categorizes tokens for the parser and tools.

- `Whitespace`: Ignored by most shapes, but preserved in the tree.
- `Comment`: Ignored by most shapes.
- `String`: A string literal.
- `Number`: A number literal.
- `Boolean`: `true` or `false`.
- `Null`: `null` or `nil`.
- `Identifier(VariableRole)`: A name. Carries a `VariableRole` (Binding, Reference, None).
- `Operator`: A symbol like `+`, `-`, `=`, etc.
- `Keyword`: A reserved word.
- `Error`: A token that failed to parse correctly.

## `define_atom!` Macro

The `define_atom!` macro simplifies implementing this trait.

```rust
# use mcparse::{define_atom, AtomKind, token::{Token, Cursor}, highlighter::{Highlighter, HighlightStyle}};
define_atom! {
    struct MyAtom;
    kind = AtomKind::String;
    parse(input) { None }
    highlight(token, highlighter) { }
}
```
