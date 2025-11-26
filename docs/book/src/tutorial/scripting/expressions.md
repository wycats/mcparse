# Expressions

Parsing expressions (like `1 + 2 * 3`) is notoriously difficult because of **operator precedence** and **associativity**.

McParse adopts a flexible **Expression Continuation** model.

## The Concept

Instead of a rigid precedence table, McParse thinks of expressions in terms of "Heads" and "Continuations".

1.  **Head**: The start of an expression (e.g., a number `1`, a variable `x`, or a parenthesized group `( ... )`).
2.  **Continuation**: Something that _extends_ an existing expression (e.g., `+ 2`, `.method()`, `[index]`).

The parser loop looks like this:

1.  Parse a **Head**.
2.  Look at the next token. Does it start a **Continuation**?
    - If yes, parse the continuation and combine it with the current expression. Repeat.
    - If no, stop.

## Implementing Simple Expressions

For our MiniScript, let's support simple addition.

```rust
# use mcparse::{shape::{Shape, term, ParseError}, token::{TokenStream, TokenTree}, MatchContext, MatchResult, AtomKind};
// Pseudo-code for an expression shape
#[derive(Debug)]
struct Expression;

impl Shape for Expression {
    fn match_shape<'a>(&self, stream: TokenStream<'a>, context: &mut dyn MatchContext) -> MatchResult<'a> {
        // 1. Parse the Head (e.g., a Number)
        let (mut lhs, mut rest) = term(AtomKind::Number).match_shape(stream, context)?;

        // 2. Loop to check for Continuations
        loop {
            // Check if the next token is a "+" operator
            let op_shape = term("+");
            if let Ok((op, after_op)) = op_shape.match_shape(rest.clone(), context) {
                // It is! Now parse the right-hand side (another Number)
                let (rhs, after_rhs) = term(AtomKind::Number).match_shape(after_op, context)?;

                // Combine lhs, op, and rhs into a new TokenTree
                lhs = TokenTree::Group(vec![lhs, op, rhs]);
                rest = after_rhs;
            } else {
                // Not a continuation, we are done.
                break;
            }
        }

        Ok((lhs, rest))
    }
}
```

## Why this model?

This model is powerful because it allows **Macros** to participate in expression parsing.

A macro can define a new operator (like `+=` or a custom `infix` operator) simply by providing a shape that acts as a continuation. The parser doesn't need to know about all operators upfront.

## Handling Precedence

To handle precedence (e.g., `*` binds tighter than `+`), you can pass a "binding power" or "precedence level" into your expression parser.

When checking for a continuation, you only accept it if its precedence is higher than the current context. This is the essence of **Pratt Parsing**, which fits perfectly with the Continuation model.
