# Core Concepts

Before diving into code, it's important to understand the mental model behind McParse. It differs significantly from traditional parser generators like Lex/Yacc or parser combinators like Nom.

## The Pipeline

Parsing in McParse happens in three distinct phases:

1.  **Atomic Lexing**: The raw text is converted into a tree of `TokenTree`s.
    - _Variable Binding_: During this phase, identifiers are classified as "bindings" (declarations) or "references" (usages) based on local context.
2.  **Macro Expansion**: The parser traverses the `TokenTree`, identifying and expanding macros. This allows user-defined syntax (like `if`, `let`, or infix operators) to transform the tree.
3.  **Shape Matching**: A grammar (defined by "Shapes") is matched against the expanded tree to produce a result (AST, error, etc.).

<div class="mermaid">
graph LR
    A[Source Code] -->|Lexer + Binding| B[Token Tree]
    B -->|Macro Expansion| C[Expanded Tree]
    C -->|Shape| D[Match Result]
</div>

### Visualizing the Pipeline

Let's trace a simple expression: `a + b * c`.

**1. Source Code**

```text
a + b * c
```

**2. Token Tree (Lexing)**
The lexer produces a flat list of tokens (since there are no parentheses).

```text
[ Identifier(a), Operator(+), Identifier(b), Operator(*), Identifier(c) ]
```

**3. Expanded Tree (Macro Expansion)**
The macro expander handles operator precedence. Since `*` binds tighter than `+`, it groups `b * c` first.

```text
[
  Group(
    Identifier(a),
    Operator(+),
    Group(
      Identifier(b),
      Operator(*),
      Identifier(c)
    )
  )
]
```

**4. Match Result (Shape Matching)**
Finally, your shapes consume this structure to produce your AST.

```rust
# struct BinaryOp<L, R> { op: Op, lhs: L, rhs: R }
# enum Op { Add, Mul }
# use Op::*;
# struct Variable(&'static str);
let _ = BinaryOp {
    op: Add,
    lhs: Variable("a"),
    rhs: BinaryOp {
        op: Mul,
        lhs: Variable("b"),
        rhs: Variable("c"),
    }
};
```

## The Token Tree

Most parsers produce a flat stream of tokens:

`if`, `(`, `x`, `)`, `{`, `y`, `}`

McParse produces a **Tree**:

- `if` (Keyword)
- `(...)` (Delimited Group)
  - `x` (Identifier)
- `{...}` (Delimited Group)
  - `y` (Identifier)

### Why a Tree?

1.  **Robustness**: The parser _cannot_ see unbalanced parentheses. The lexer guarantees that if you see a group, it is balanced.
2.  **Macros**: A macro can consume a whole group (like `{ ... }`) as a single item, without needing to parse its contents. This is how Rust macros work, and it's why McParse is so extensible.
3.  **Error Recovery**: If there is a syntax error inside a group, we can skip the whole group and continue parsing the next item.

## Shape Algebra

"Shapes" are the language you use to describe your grammar. They are combinators that operate on the `TokenTree`.

- **`term`**: Matches a single leaf node (like a keyword or identifier).
- **`enter`**: "Dives" into a delimited group to parse its contents.
- **`seq`**: Matches one shape after another.
- **`choice`**: Tries one shape, then another.

Because Shapes operate on trees, they are naturally recursive and composable.
