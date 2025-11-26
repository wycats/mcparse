# Custom Atoms

While `RegexAtom` and `KeywordAtom` cover 90% of use cases, sometimes you need more control. McParse allows you to implement the `Atom` trait manually for complex lexing logic.

## When to use Custom Atoms

1.  **Complex State**: Parsing Python-style indentation or Haskell layout rules.
2.  **Performance**: Hand-written state machines can be faster than regex for very specific patterns.
3.  **Context Sensitivity**: Tokens that depend on external state or lookahead beyond what regex allows.

## The `Atom` Trait

```rust
# use std::fmt::Debug;
# use mcparse::{AtomKind, Token, Cursor, Highlighter};
pub trait Atom: Debug + Send + Sync {
    /// Returns the kind of token this atom produces.
    fn kind(&self) -> AtomKind;

    /// Tries to parse a token from the current cursor position.
    /// Returns `Some((Token, Cursor))` if successful, or `None` if the input doesn't match.
    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)>;

    /// Applies syntax highlighting to the token.
    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter);
}
```

## Example: A Custom String Lexer

Let's implement a string lexer that handles escaped characters manually, which might be hard to do with a single regex.

```rust
use mcparse::{atom::{Atom, AtomKind}, token::{Token, SourceLocation, Cursor}, highlighter::{Highlighter, HighlightStyle}};
use std::fmt::Debug;

#[derive(Debug)]
struct CustomString;

impl Atom for CustomString {
    fn kind(&self) -> AtomKind {
        AtomKind::String
    }

    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)> {
        if !input.rest.starts_with('"') {
            return None;
        }

        let mut len = 1; // Skip opening quote
        let mut escaped = false;

        for c in input.rest[1..].chars() {
            len += c.len_utf8();
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                // Found closing quote
                return Some((
                    Token {
                        kind: AtomKind::String,
                        text: input.rest[..len].to_string(),
                        location: SourceLocation {
                            span: (input.offset, len).into(),
                        },
                        atom_index: None,
                    },
                    input.advance(len),
                ));
            }
        }

        // EOF before closing quote
        None
    }

    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        highlighter.highlight(token, HighlightStyle::String);
    }
}
```

## Using Custom Atoms

You can mix custom atoms with declarative ones in your language definition:

```rust
# use mcparse::{define_language, language::{Delimiter, VariableRules, VariableRole, Language}, atom::{Atom, AtomKind}, token::{Token, Cursor, SourceLocation}, highlighter::{Highlighter, HighlightStyle}};
# #[derive(Debug)] struct CustomString;
# impl Atom for CustomString { fn kind(&self) -> AtomKind { AtomKind::String } fn parse<'a>(&self, i: Cursor<'a>) -> Option<(Token, Cursor<'a>)> { None } fn highlight(&self, t: &Token, h: &mut dyn Highlighter) {} }
# #[derive(Debug)] struct NoOpVariableRules;
# impl VariableRules for NoOpVariableRules { fn classify(&self, _p: Option<&Token>, _c: &Token) -> VariableRole { VariableRole::None } }

define_language! {
    struct MyLang;
    atoms = [
        // Use our custom atom
        CustomString,
        // Mix with declarative atoms (once the macro supports it)
        // atom Whitespace = r"\s+";
    ];
    delimiters = [];
    variable_rules = NoOpVariableRules;
}
```
