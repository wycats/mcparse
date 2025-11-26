# Infix Operators

For simple binary expressions, you can use the "Continuation" pattern.

```rust
// See "Tutorial: Scripting Language" for the full implementation.
// This is the quick recipe.

struct Expression;
impl Shape for Expression {
    fn match_shape(&self, stream: TokenStream, context: &mut dyn MatchContext) -> MatchResult {
        let (mut lhs, mut rest) = Term.match_shape(stream, context)?;

        loop {
            // Check for operator
            if let Ok((op, after_op)) = Operator.match_shape(rest, context) {
                // Parse RHS
                let (rhs, after_rhs) = Term.match_shape(after_op, context)?;
                // Combine
                lhs = make_binary_node(lhs, op, rhs);
                rest = after_rhs;
            } else {
                break;
            }
        }
        Ok((lhs, rest))
    }
}
```
