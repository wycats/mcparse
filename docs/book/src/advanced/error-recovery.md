# Error Recovery

In a batch compiler, it's acceptable to stop at the first error. In an IDE, it is **unacceptable**. The user spends most of their time writing incomplete or invalid code, and they still expect syntax highlighting and autocompletion to work.

McParse provides a declarative way to handle errors using the `recover` combinator.

## The Strategy: Panic Mode Recovery

The most common recovery strategy is "Panic Mode": when an error occurs, skip tokens until you find a "synchronization point" (like a semicolon `;` or a closing brace `}`), and then try to resume parsing.

## The `recover` Combinator

McParse encapsulates this logic in the `recover(shape, terminator)` combinator.

```rust
use mcparse::{shape::{recover, term, seq, Matcher}, token::TokenTree, AtomKind};

# use mcparse::shape::{Shape, MatchContext, MatchResult, ParseError};
# use mcparse::token::TokenStream;
# #[derive(Debug, Clone)] struct Expression;
# impl Shape for Expression { fn match_shape<'a>(&self, s: TokenStream<'a>, _: &mut dyn MatchContext) -> MatchResult<'a> { Err(ParseError::new((0,0).into(), "".into())) } }
# #[derive(Debug, Clone)] struct AnyIdentifier;
# impl Matcher for AnyIdentifier { fn matches(&self, t: &TokenTree) -> bool { matches!(t, TokenTree::Token(tok) if matches!(tok.kind, AtomKind::Identifier)) } fn describe(&self) -> String { "identifier".into() } }

// Parse a statement: "let" <ident> "=" <expr> ";"
let statement = seq(
    term("let"),
    seq(term(AnyIdentifier), seq(term("="), seq(Expression, term(";"))))
);

// Wrap it in recover!
// If `statement` fails to match, we skip tokens until we see a semicolon ";".
let safe_statement = recover(statement, ";");
```

## How it Works

1.  The parser attempts to match `statement`.
2.  If it succeeds, great!
3.  If it fails (returns `Err`), `recover` catches the error.
4.  It enters a loop, consuming tokens one by one.
5.  In each iteration, it checks if the `terminator` (here, `term(";")`) matches.
6.  If the terminator matches, it consumes it and returns a `TokenTree::Error` containing the skipped tokens.
7.  The parent shape (e.g., a list of statements) sees a successful match (of an Error node) and continues to the next statement.

This ensures that a syntax error in one statement doesn't cascade and break the parsing of the rest of the file.
