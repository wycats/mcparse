# Decision Log

This file tracks key architectural and design decisions made throughout the project. It serves as a reference to understand _why_ things are the way they are and prevents re-litigating settled issues without new information.

## Format

### [Date] Title of Decision

- **Context**: What was the problem or situation?
- **Decision**: What did we decide to do?
- **Rationale**: Why did we choose this path? What alternatives were considered?

### [2025-11-25] Expression Continuation Model

- **Context**: We needed a way to handle operator precedence and expression parsing that is flexible enough for macros to extend.
- **Decision**: Adopted an "Expression Continuation" model where the parser parses a "head" term and then repeatedly checks if the next token (e.g., an operator) wants to "continue" the expression.
- **Rationale**: This allows macros to participate in the expression parsing loop dynamically, enabling powerful DSLs and custom operators without a rigid, pre-computed precedence table.

### [2025-11-25] Atomic Whitespace

- **Context**: We needed to handle whitespace in a way that supports both "loose" parsing (ignoring whitespace) and "tight" parsing (e.g., method calls `obj.method` vs `obj . method`).
- **Decision**: Treat whitespace as a distinct `Atom` during the initial lexing phase, but provide Shape Algebra primitives (`term`, `enter`) that implicitly skip it, while offering `adjacent` for explicit whitespace checks.
- **Rationale**: This preserves whitespace information for high-fidelity tools (formatters, highlighters) while keeping the common case of parsing (ignoring whitespace) ergonomic for the grammar writer.

### [2025-11-26] Token Tree Model

- **Context**: We needed to handle delimited groups (parentheses, braces) robustly, especially for macros that might consume arbitrary tokens inside a group.
- **Decision**: The lexer produces a tree of `TokenTree` nodes (Token or Delimited Group) rather than a flat stream. Shapes like `enter` are used to navigate into these groups.
- **Rationale**: This simplifies macro argument parsing (a macro can consume a whole group as one tree) and ensures that delimiters are always balanced by construction in the parser view.

### [2025-11-26] Variable Role in AtomKind

- **Context**: We needed to distinguish between variable declarations (bindings) and references to support hygiene and semantic highlighting, without a separate semantic analysis pass.
- **Decision**: `AtomKind::Identifier` carries a `VariableRole` enum (Binding, Reference, None). A `VariableRules` trait allows the language to classify identifiers during the atomic lexing phase based on local context (e.g., "after `let`").
- **Rationale**: This allows the parser and highlighter to have immediate access to binding information, enabling "syntax-directed" hygiene and richer highlighting even in broken code.

### [2025-11-26] Concrete Shape Types

- **Context**: Derived combinators like `separated` need to clone their argument shapes to repeat them. However, returning opaque `impl Shape` from constructors hides the `Clone` implementation.
- **Decision**: Refactored `src/shape.rs` to return concrete generic structs (e.g., `Seq<A, B>`) from all constructor functions and derived `Clone` on them.
- **Rationale**: This enables the composition of complex, reusable shapes without boxing everything, maintaining zero-cost abstractions where possible while satisfying trait bounds like `Clone`.

### [2025-11-26] Error Recovery via `recover` Combinator

- **Context**: The parser needs to be able to recover from syntax errors and continue parsing subsequent statements to provide a good IDE experience.
- **Decision**: Introduced a `recover(shape, terminator)` combinator that attempts to match a shape and, on failure, skips tokens until a terminator is found, returning a `TokenTree::Error`.
- **Rationale**: This provides a declarative way to specify synchronization points (like semicolons or closing braces) directly in the grammar, keeping the error recovery logic local to the relevant shapes.
