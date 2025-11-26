# Tutorial: Building a Scripting Language

In this tutorial, we will build "MiniScript", a small dynamic language with variables, functions, and expressions.

While the JSON tutorial focused on static data structures, this tutorial will demonstrate McParse's unique features for **programming languages**:

- **Macros**: How to define keywords like `let` or `if` as macros that extend the language syntax.
- **Hygiene**: How to handle variable scopes and bindings correctly using `VariableRules`.
- **Expressions**: How to parse expressions with operators using a flexible continuation model.

## The Goal

We want to parse code like this:

```rust
# fn print(_: i32) {}
let x = 42;
let y = x + 10;
print(y);
```

And we want the parser to understand that:

1.  `let` is a keyword that introduces a variable binding.
2.  `x` in `let x` is a **Binding** (declaration).
3.  `x` in `x + 10` is a **Reference** (usage).
4.  `=` and `+` are operators.

Let's get started!
