# Shape Combinators

Shapes are the building blocks of your grammar. They are combinators that match patterns in the `TokenTree`.

## Basic Shapes

- `term(kind)`: Matches a single token of the given `AtomKind`.
- `term("text")`: Matches a single token with the exact text.

## Composition

- `seq(a, b)`: Matches `a` followed by `b`.
- `choice(a, b)`: Tries to match `a`. If it fails, tries to match `b`.
- `rep(a)`: Matches `a` zero or more times.
- `opt(a)`: Matches `a` zero or one time.

## Structure

- `enter(delimiter, inner)`: Matches a delimited group (e.g., `{ ... }`) and parses its contents using `inner`.
- `separated(item, sep)`: Matches a list of `item`s separated by `sep`.

## Error Handling

- `recover(shape, terminator)`: Tries to match `shape`. If it fails, skips tokens until `terminator` is found.

## Whitespace Handling

- `adjacent(a, b)`: Matches `a` followed immediately by `b` with **no whitespace** in between.
