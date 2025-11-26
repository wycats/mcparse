# Tutorial: Building a JSON Parser

In this tutorial, we will build a fully compliant JSON parser using McParse. JSON is a great starting point because it has a simple, recursive structure but still requires handling strings, numbers, and nested objects/arrays.

We will cover:

1.  **Defining Atoms**: How to define the basic tokens of your language (strings, numbers, booleans, null).
2.  **Defining Shapes**: How to use Shape Algebra to define the grammar (Objects, Arrays, Key-Value pairs).
3.  **Handling Delimiters**: How to use `TokenTree`s to parse nested structures like `{ ... }` and `[ ... ]`.

By the end of this tutorial, you will have a working JSON parser that can handle real-world data and produce meaningful error messages.

## Prerequisites

Make sure you have `mcparse` added to your `Cargo.toml`:

```toml
[dependencies]
mcparse = "0.1.0"
```
