# Defining Atoms

In McParse, the first step is to define the "Atoms" of your language. These are the raw tokens that the lexer will identify.

For JSON, we need:

1.  **Whitespace**: Spaces, tabs, newlines.
2.  **Strings**: `"hello"`.
3.  **Numbers**: `123`, `3.14`.
4.  **Booleans**: `true`, `false`.
5.  **Null**: `null`.
6.  **Punctuation**: `:`, `,`.

## The `Atom` Trait

Every atom must implement the `Atom` trait. This trait tells the lexer how to recognize the atom and what kind of token it produces.

You can implement it manually, but McParse provides a `define_atom!` macro to make it easier.

### Whitespace

Whitespace is crucial for separating tokens, even if we often ignore it in the grammar.

```rust
use mcparse::{define_atom, atom::{Atom, AtomKind}, token::{Token, SourceLocation, Cursor}, highlighter::{Highlighter, HighlightStyle}};

define_atom! {
    struct Whitespace;
    // The kind of token this atom produces.
    kind = AtomKind::Whitespace;

    // The parsing logic.
    // `input` is a `Cursor` pointing to the current position in the source.
    parse(input) {
        let mut len = 0;
        // Iterate over characters starting from the current position.
        for c in input.rest.chars() {
            if c.is_whitespace() {
                len += c.len_utf8();
            } else {
                break;
            }
        }

        // If we found any whitespace...
        if len > 0 {
            Some((
                // Create the token.
                Token {
                    kind: AtomKind::Whitespace,
                    text: input.rest[..len].to_string(),
                    location: SourceLocation {
                        span: (input.offset, len).into(),
                    },
                },
                // Return the token and the advanced cursor.
                input.advance(len),
            ))
        } else {
            // No match.
            None
        }
    }

    // How to highlight this token in the IDE.
    highlight(token, highlighter) {
        highlighter.highlight(token, HighlightStyle::None);
    }
}
```

### Strings

We need to match double-quoted strings.

```rust
# use mcparse::{define_atom, atom::{Atom, AtomKind}, token::{Token, SourceLocation, Cursor}, highlighter::{Highlighter, HighlightStyle}};
define_atom! {
    struct StringLiteral;
    kind = AtomKind::String;
    parse(input) {
        // Check if the input starts with a double quote.
        if input.rest.starts_with('"') {
            let mut len = 1;
            let mut escaped = false;

            // Iterate over the rest of the string to find the closing quote.
            for c in input.rest[1..].chars() {
                len += c.len_utf8();
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    // Found the closing quote!
                    return Some((
                        Token {
                            kind: AtomKind::String,
                            text: input.rest[..len].to_string(),
                            location: SourceLocation {
                                span: (input.offset, len).into(),
                            },
                        },
                        input.advance(len),
                    ));
                }
            }
        }
        // Did not match a complete string.
        None
    }
    highlight(token, highlighter) {
        highlighter.highlight(token, HighlightStyle::String);
    }
}
```

### Punctuation

For simple static strings like `:` and `,`, we can create a reusable struct.

```rust
# use mcparse::{atom::{Atom, AtomKind}, token::{Token, SourceLocation, Cursor}, highlighter::{Highlighter, HighlightStyle}};
#[derive(Debug)]
struct Punctuation(String);

impl Atom for Punctuation {
    fn kind(&self) -> AtomKind {
        AtomKind::Operator
    }
    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)> {
        if input.rest.starts_with(&self.0) {
            Some((
                Token {
                    kind: AtomKind::Operator,
                    text: self.0.clone(),
                    location: SourceLocation {
                        span: (input.offset, self.0.len()).into(),
                    },
                },
                input.advance(self.0.len()),
            ))
        } else {
            None
        }
    }
    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        highlighter.highlight(token, HighlightStyle::Operator);
    }
}
```

## Defining the Language

Now we combine these atoms into a `Language` definition. This is also where we define **Delimiters**.

Delimiters (like `{}` and `[]`) are special. The lexer matches them and creates a recursive `TokenTree` structure. You don't define them as Atoms; you define them in the `delimiters` section of `define_language!`.

```rust
# use mcparse::{define_atom, atom::{Atom, AtomKind}, token::{Token, SourceLocation, Cursor}, highlighter::{Highlighter, HighlightStyle}};
# define_atom! { struct Whitespace; kind = AtomKind::Whitespace; parse(input) { None } highlight(token, h) {} }
# define_atom! { struct StringLiteral; kind = AtomKind::String; parse(input) { None } highlight(token, h) {} }
# #[derive(Debug)] struct Punctuation(String);
# impl Atom for Punctuation { fn kind(&self) -> AtomKind { AtomKind::Operator } fn parse<'a>(&self, i: Cursor<'a>) -> Option<(Token, Cursor<'a>)> { None } fn highlight(&self, t: &Token, h: &mut dyn Highlighter) {} }
use mcparse::{define_language, language::{Delimiter, VariableRules, VariableRole}};

#[derive(Debug)]
struct NoOpVariableRules;
impl VariableRules for NoOpVariableRules {
    fn classify(&self, _prev: Option<&Token>, _curr: &Token) -> VariableRole {
        VariableRole::None
    }
}

define_language! {
    struct JsonLang;
    atoms = [
        Whitespace,
        Punctuation(":".into()),
        Punctuation(",".into()),
        StringLiteral,
        // Add NumberLiteral, Boolean, Null here...
    ];
    delimiters = [
        Delimiter {
            kind: "brace",
            open: "{",
            close: "}",
        },
        Delimiter {
            kind: "bracket",
            open: "[",
            close: "]",
        },
    ];
    variable_rules = NoOpVariableRules;
}
```

With this, we have a working lexer! In the next section, we'll use these atoms to define the grammar shapes.
