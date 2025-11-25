# Implementation Plan: Core Parsing Engine & Semantics

## Goal

Implement the core logic of McParse, including the shape algebra, macro expansion loop, variable binding/hygiene, and error recovery.

## Steps

1.  **Shape Algebra**: Implement the combinators defined in `DESIGN.md` (`seq`, `choice`, `rep`, `opt`, `adjacent`, `separated`).
2.  **Adjacency & Whitespace**: Implement the logic to enforce adjacency constraints and handle whitespace atoms correctly within shapes.
3.  **Variable Binding**: Implement the `VariableRules` trait and integrate it into the atomic lexing phase to identify bindings and references.
4.  **Macro Expansion**: Implement the main parsing loop that consumes tokens, identifies macros (respecting shadowing), and expands them.
5.  **Error Recovery**: Implement "forgiving" parsing logic in shapes (e.g., skipping to delimiters) to handle malformed input gracefully.
6.  **Integration Testing (JsonPlus)**: Use the "JsonPlus" aspirational language as the primary integration test.
    - Scaffold `examples/json_plus.rs` early.
    - As we implement shape combinators, use them to define `JsonPlus` shapes.
    - Verify parsing of JSON constructs (objects, arrays) to test recursion and delimiters.
    - Verify `import` macro to test macro expansion and binding.
