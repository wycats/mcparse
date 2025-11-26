# API Ergonomics & Philosophy

## The Goal: "Magical" Composition

The core value proposition of `mcparse` is that it gives you "magical" features (resilience, incrementalism, rich IDE support) for free if you play by its rules (Shape Algebra, Atomic Lexing).

However, the current API can feel verbose for simple tasks. A user coming from `lalrpop` or `pest` might look at our `Atom` implementation and ask: "Why can't I just use a regex for whitespace?"

Our goal for API refinement is to bridge this gap without creating a "cliff".

## Core Constraints

1.  **No "Easy Bake Oven"**: We don't want a simplified "mode" that you have to graduate from. The "easy" way should just be a concise way to generate the "hard" way.
2.  **Seamless Composition**: Using a high-level helper (like a regex-based Atom) must not prevent you from dropping down to a manual implementation for a neighboring part of the grammar.
3.  **Transparent Magic**: The user should feel that the library is doing heavy lifting (incrementalism, error recovery) _for_ them, not _hiding_ things from them.

## The "Easy" Cases

We want to make the following common patterns trivial:

- **Keywords & Literals**: Should be declarative.
- **Whitespace**: Should be definable via simple rules (e.g., "standard C-style whitespace") or regex, without writing a state machine.
- **Identifiers**: Should be definable via regex or character classes.

## The "Hard" Cases

We want to ensure the following remains possible and integrates smoothly:

- **Context-Sensitive Lexing**: e.g., "Soft keywords" that are only keywords in certain positions.
- **Complex State**: e.g., Python-style indentation tracking (which requires state in the lexer).
- **Custom Error Recovery**: Fine-tuning exactly where the parser re-synchronizes.

## Example: Declarative Macros with Escape Hatches

The `define_language!` macro illustrates this philosophy by allowing users to mix declarative rules with custom implementations.

```rust
define_language! {
    name = MyLang;

    // Easy case: Regex-based atoms
    atom Whitespace = r"\s+";
    atom Identifier = r"[a-zA-Z_][a-zA-Z0-9_]*";

    // Medium case: Keywords (declarative list)
    keywords = ["if", "else", "fn", "let"];

    // Hard case: Custom implementation reference
    atom StringLiteral = custom::StringLexer;
}
```

### Expansion

The macro above would generate code similar to this:

````rust
struct MyLang {
    atoms: Vec<Box<dyn Atom>>,
    // ...
}

impl Language for MyLang {
    fn atoms(&self) -> &[Box<dyn Atom>] {
        &self.atoms
    }
    // ...
}

impl MyLang {
    fn new() -> Self {
        Self {
            atoms: vec![
                // Regex atoms use a helper struct
                Box::new(mcparse::atoms::RegexAtom::new(
                    AtomKind::Whitespace,
                    r"\s+"
                )),
                Box::new(mcparse::atoms::RegexAtom::new(
                    AtomKind::Identifier,
                    r"[a-zA-Z_][a-zA-Z0-9_]*"
                )),
                // Keywords are often handled by a specialized atom
                Box::new(mcparse::atoms::KeywordAtom::new(&[
                    "if", "else", "fn", "let"
                ])),
                // Custom atoms are instantiated directly
                Box::new(custom::StringLexer),
            ],
            // ...
        }
    }
}

### Under the Hood: RegexAtom

The `RegexAtom` struct handles the boilerplate of compiling the regex and matching it against the input. Note that `Atom`s are responsible for *lexing* and *highlighting*, while *completion* is handled by the `Shape` algebra (specifically `Matcher`s).

```rust
pub struct RegexAtom {
    kind: AtomKind,
    regex: regex::Regex,
}

impl RegexAtom {
    pub fn new(kind: AtomKind, pattern: &str) -> Self {
        // Ensure the regex matches from the start of the string
        let pattern = if pattern.starts_with('^') {
            pattern.to_string()
        } else {
            format!("^{}", pattern)
        };

        Self {
            kind,
            regex: regex::Regex::new(&pattern).expect("Invalid regex"),
        }
    }
}

impl Atom for RegexAtom {
    fn kind(&self) -> AtomKind {
        self.kind.clone()
    }

    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)> {
        // Regex matches against the remaining string
        if let Some(mat) = self.regex.find(input.rest) {
            let len = mat.end(); // Match length
            Some((
                Token {
                    kind: self.kind.clone(),
                    text: input.rest[..len].to_string(),
                    location: SourceLocation {
                        span: (input.offset, len).into(),
                    },
                },
                input.advance(len),
            ))
        } else {
            None
        }
    }

    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        // Default highlighting based on kind
        let style = match self.kind {
            AtomKind::String => HighlightStyle::String,
            AtomKind::Number => HighlightStyle::Number,
            AtomKind::Operator => HighlightStyle::Operator,
            AtomKind::Whitespace => HighlightStyle::None,
            _ => HighlightStyle::None,
        };
        highlighter.highlight(token, style);
    }
}
````
