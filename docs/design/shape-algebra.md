# Shape Algebra

The Shape Algebra defines a set of composable primitives for matching patterns in a `TokenStream`.

## Data Model: Token Trees

McParse operates on a **Token Tree**, not a flat list of tokens. This is crucial for the algebra.

- **Atom**: A leaf node (Identifier, Number, Operator, etc.).
- **Delimited Group**: A single node containing a nested `TokenStream`. E.g., `( a b c )` is **one** `TokenTree` item in the parent stream, which contains a child stream `a b c`.
- **Whitespace**: Represented as Atoms, but usually skipped.

## Invariants

1.  **Leading Whitespace Skipping**: The `term` primitive automatically skips leading `Whitespace` atoms before attempting to match. This means `seq(a, b)` implicitly skips whitespace between `a` and `b` because `b`'s first `term` will skip it.
2.  **Adjacency**: To enforce "no whitespace", we must explicitly check the stream between matches (handled by `adjacent`).
3.  **Tree Navigation**: To match inside a group (like parens), we must explicitly `enter` that group. We cannot match the open delimiter, then contents, then close delimiter as a sequence, because they are structurally one node.
4.  **Error Propagation**: Shapes return `Result<..., ParseError>`. If a shape fails to match, it returns a structured error containing a `SourceSpan` and a descriptive message. Combinators like `seq` propagate the first error encountered. `choice` suppresses the error from the first branch if it fails, trying the second branch instead.

## Error Handling

The Shape Algebra produces rich, structured errors natively.

- **`ParseError`**: Contains a `SourceSpan` (location) and a `message` (String).
- **`Matcher::describe`**: All matchers must implement `describe()` to provide human-readable names for expected tokens (e.g., "Identifier", "Delimiter '{'").
- **Automatic Errors**: The `term` primitive automatically generates errors like "Expected Identifier, found Number" when a match fails.
- **Contextual Errors**: Combinators like `enter` generate errors if the inner shape does not consume the entire group content ("Expected end of group").

## Primitives

### `term(matcher)`

- **Input**: `TokenStream`
- **Parameters**: `matcher: impl Matcher`
- **Behavior**:
  1. Skips leading `Whitespace` atoms.
  2. Checks if the next `TokenTree` satisfies the `matcher`.
- **Matcher Types**:
  - `AtomKind`: Matches a `Token` of this kind.
  - `&str`: Matches a `Token` with this exact text.
  - `Delimiter`: Matches a `Delimited` token tree with this delimiter type (without entering it).
- **Success**: Returns the matched `TokenTree`. Consumes 1 item (plus skipped whitespace).
- **Failure**: Returns `ParseError` describing what was expected vs. what was found.

### `seq(a, b)`

- **Input**: `TokenStream`
- **Parameters**: `a: Shape`, `b: Shape`
- **Behavior**: Matches `a`, then matches `b` against the remaining stream.
- **Whitespace**: Implicitly allows whitespace between `a` and `b` (due to `term`'s behavior).
- **Failure**: Returns the error from `a` if `a` fails, or the error from `b` if `b` fails.

### `choice(a, b)`

- **Input**: `TokenStream`
- **Parameters**: `a: Shape`, `b: Shape`
- **Behavior**: Tries `a`. If it fails, tries `b`.
- **Failure**: If `a` fails, the error is discarded and `b` is attempted. If `b` also fails, `b`'s error is returned.

### `rep(a)`

- **Input**: `TokenStream`
- **Parameters**: `a: Shape`
- **Behavior**: Repeatedly matches `a` zero or more times.
- **Whitespace**: Implicitly allows whitespace between repetitions.
- **Failure**: `rep` generally succeeds (matching 0 times). It stops matching when `a` fails.

### `enter(delimiter, inner)`

- **Input**: `TokenStream`
- **Parameters**: `delimiter: Delimiter`, `inner: Shape`
- **Behavior**:
  1. Matches a `TokenTree::Delimited` with the given `delimiter` (skipping leading whitespace).
  2. Creates a _new_ `TokenStream` from the content of that delimited group.
  3. Matches `inner` against that inner stream.
  4. Ensures `inner` consumes the entire group content (implicit `end()`).
- **Success**: Returns the result of `inner`.
- **Failure**: Returns error if delimiter doesn't match, if `inner` fails, or if `inner` leaves unconsumed tokens.

### `adjacent(a, b)`

- **Input**: `TokenStream`
- **Parameters**: `a: Shape`, `b: Shape`
- **Behavior**:
  1. Matches `a`.
  2. Peeks at the _raw_ next token (no skipping). If it is `Whitespace`, fail.
  3. Matches `b`.
- **Failure**: Returns "Unexpected whitespace" error if whitespace is found between `a` and `b`.

## Derived Combinators

### `opt(a)`

- Definition: `choice(a, empty())`

### `separated(item, sep)`

- Definition: `seq(item, rep(seq(sep, item)))`

### `joined(a)`

- Definition: `seq(a, rep(adjacent(a)))` (Conceptual)
- Matches `a` repeated one or more times with **no** intervening whitespace.
