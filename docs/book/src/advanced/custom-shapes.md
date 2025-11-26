# Custom Shapes

While McParse provides many built-in combinators (`seq`, `choice`, `rep`, etc.), sometimes you need custom parsing logic. You can achieve this by implementing the `Shape` trait directly.

## The `Shape` Trait

```rust
# use std::fmt::Debug;
# use mcparse::{token::TokenStream, MatchContext, MatchResult};
pub trait Shape: Debug {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
    ) -> MatchResult<'a>;
}
```

- `stream`: The current position in the token stream.
- `context`: Provides access to shared state (like the string interner or error reporter).
- `MatchResult`: `Result<(TokenTree, TokenStream<'a>), ParseError>`.

## Example: Matching a Specific Contextual Keyword

Suppose you want to match the identifier `contextual` but only if it appears in a specific place.

```rust
use mcparse::{
    shape::{Shape, MatchContext, MatchResult, ParseError},
    token::{TokenStream, TokenTree},
    AtomKind,
};

#[derive(Debug, Clone)]
struct ContextualKeyword;

impl Shape for ContextualKeyword {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        _context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        // 1. Peek at the next token
        if let Some(TokenTree::Token(token)) = stream.first() {
             // 2. Check if it matches our criteria
            if let AtomKind::Identifier = token.kind {
                if token.text == "contextual" {
                    // 3. Success! Return the token tree and the ADVANCED stream.
                    return Ok((TokenTree::Token(token.clone()), stream.advance(1)));
                }
            }

            // 4. Failure. Return an error.
            return Err(ParseError::new(
                token.location.span,
                format!("Expected keyword 'contextual', found {}", token.text),
            ));
        }

        Err(ParseError::new(
            (0, 0).into(),
            "Unexpected EOF".into(),
        ))
    }
}
```

## Best Practices

1.  **Don't Panic**: Always return `Err(ParseError)` on failure.
2.  **Advance the Stream**: On success, make sure to return the `next_stream` that points _after_ the consumed tokens.
3.  **Use `MatchContext`**: If you need to store state or look up symbols, use the `context`.
