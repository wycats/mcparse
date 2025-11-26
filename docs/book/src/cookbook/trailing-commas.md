# Trailing Commas

It is common to allow trailing commas in lists (e.g., `[1, 2, ]`).

The `separated` combinator does _not_ allow trailing separators by default. To support them, you can combine `separated` with an optional separator at the end.

```rust
// List of items separated by commas
let list = separated(Item, term(","));

// Allow optional trailing comma
let list_with_trailing = seq(list, opt(term(",")));
```

Alternatively, if you are writing a custom loop, you can just peek for the separator after each item.
