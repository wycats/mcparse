# Implementation Plan: Core Parsing Engine & Semantics

## Goal

Implement the core logic of McParse, including the shape algebra, macro expansion loop, variable binding/hygiene, and error recovery.

## Steps

1.  **Shape Algebra**: Implement the core primitives and combinators defined in `docs/design/shape-algebra.md` in `src/shape.rs`.
    - **Primitives**:
      - `term(matcher)`: Matches a single `TokenTree` (Atom, Delimiter, etc.) skipping leading whitespace.
      - `seq(a, b)`: Matches `a` then `b` (implicitly skipping whitespace).
      - `choice(a, b)`: Matches `a`, if fails, matches `b`.
      - `rep(a)`: Matches `a` zero or more times.
      - `enter(delimiter, inner)`: Matches a delimited group and matches `inner` against its content.
      - `adjacent(a, b)`: Matches `a` then `b` with **no** intervening whitespace.
      - `empty()`: Always succeeds, consumes nothing.
      - `end()`: Succeeds only at the end of the stream.
    - **Derived Combinators**:
      - `opt(a)`: `choice(a, empty())`.
      - `separated(item, sep)`: Matches items separated by a separator.
      - `joined(a)`: Matches `a` repeated with no whitespace.
2.  **Adjacency & Whitespace**: Implement the logic to enforce adjacency constraints and handle whitespace atoms correctly within shapes.
    - Ensure `TokenStream` can peek/consume whitespace correctly.
    - Update `adjacent` combinator to check for whitespace.
3.  **Variable Binding**: Implement the `VariableRules` trait and integrate it into the atomic lexing phase to identify bindings and references.
4.  **Macro Expansion**: Implement the main parsing loop that consumes tokens, identifies macros (respecting shadowing), and expands them.
5.  **Error Recovery**: Implement "forgiving" parsing logic in shapes (e.g., skipping to delimiters) to handle malformed input gracefully.
6.  **Integration Testing**:
    - **JsonPlus**: Use for testing basic shapes, recursion, and delimiters.
    - **MiniScript**: Use for testing macros, variable binding, and control flow structures.
    - Scaffold `examples/json_plus.rs` and `examples/miniscript.rs`.
