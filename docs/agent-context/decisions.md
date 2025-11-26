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

### [2025-11-27] Structured Parse Errors

- **Context**: The `Shape` trait originally returned `Result<T, ()>`, which meant parse failures carried no information about _what_ was expected.
- **Decision**: Refactored `Shape` to return `Result<T, ParseError>`. Added `Matcher::describe()` to allow shapes to self-document their expectations (e.g., "Expected Identifier").
- **Rationale**: This enables the parser to generate rich, user-friendly error messages automatically without requiring the grammar writer to manually instrument every failure point.

### [2025-11-27] Red/Green Tree Architecture for Incrementalism

- **Context**: We need a strategy for incremental parsing that supports efficient updates without re-parsing the entire file.
- **Decision**: Adopted the "Red/Green Tree" model (inspired by Roslyn/Rowan). "Green" nodes are immutable, position-independent storage. "Red" nodes are transient cursors that provide absolute offsets and parent pointers.
- **Rationale**: This separation allows for structural sharing (cheap cloning of subtrees) and efficient re-use of unchanged nodes during incremental updates.

### [2025-11-27] Tree-Based Invalidation

- **Context**: When a user edits the source code, we need to determine which parts of the parse tree are invalid.
- **Decision**: Edits will be mapped to the deepest containing node in the Green Tree. That node and its ancestors are invalidated. We will re-lex the content of the invalidated node and attempt to splice new tokens in.
- **Rationale**: This provides a coarse-grained but robust invalidation strategy that leverages the hierarchical structure of the code (e.g., re-parsing just a single function body) without complex state tracking.

### [2025-11-28] Documentation-Driven Testing

- **Context**: We needed a way to ensure that the code examples in "The McParse Book" remain correct and compile as the library evolves.
- **Decision**: Integrated book chapters directly into the crate's test suite using `#[doc = include_str!("...")]` in `src/lib.rs`.
- **Rationale**: This treats documentation as code, ensuring that `cargo test` verifies the examples. It avoids the need for complex external tooling or fragile shell scripts to extract and run code blocks.

### [2025-11-28] Playground Configuration

- **Context**: The default "Play" button in `mdbook` points to the official Rust Playground, which cannot run code depending on local/unpublished crates like `mcparse`.
- **Decision**: Disabled the `runnable` feature in `book.toml` but kept `copyable` enabled.
- **Rationale**: This prevents a broken user experience where the "Play" button would inevitably fail, while still allowing users to copy code for local experimentation.

### [2025-11-29] Declarative Atoms

- **Context**: Defining `Atom` implementations manually (state machines) was too verbose for simple cases like keywords or regex patterns.
- **Decision**: Introduced `RegexAtom` and `KeywordAtom` structs, and updated the `define_language!` macro to support a declarative syntax: `atom Name = r"..."` and `keyword "if"`.
- **Rationale**: This drastically reduces boilerplate for the 90% use case while preserving the "escape hatch" of manual `Atom` implementation for complex tokens (like string interpolation).

### [2025-11-29] Lexing-Time Binding

- **Context**: We initially conflated variable binding with macro expansion, which led to confusion about scope and timing.
- **Decision**: Clarified that variable binding is a strictly **lexing-time** property determined by `VariableRules`. Macros cannot dynamically introduce bindings; they can only manipulate tokens that were _already_ marked as bindings by the lexer.
- **Rationale**: This separation of concerns simplifies the mental model. The lexer handles "what is this?" (Binding vs Reference), and the parser/macros handle "what does this mean?" (Structure/Semantics). It also enables robust syntax highlighting without running the full parser.

### [2025-12-02] Post-Lexing Binding Pass

- **Context**: The "Lexing-Time Binding" approach (using `VariableRules` during atomic lexing) proved insufficient for block scoping and shadowing because the lexer is strictly linear and doesn't track nested scope state effectively (especially with lookahead limitations).
- **Decision**: Replaced `VariableRules` with a multi-pass architecture: 1. Atomic Lexing (produces `TokenTree`), 2. `BindingPass` (identifies declarations), 3. `ReferencePass` (resolves references), 4. Parsing.
- **Rationale**: This decouples the "what is a token" (Lexer) from "what does it mean in context" (Scoping). It allows for full tree traversal (handling nested blocks) and lookahead/lookbehind without complicating the core lexer. It also enables correct handling of shadowing and forward references (if desired).

### [2025-12-02] Red/Green Tree Architecture

- **Context**: To support efficient incremental updates, we need a tree structure that allows structural sharing (reusing unchanged parts of the tree). The existing `TokenTree` stored absolute offsets, making it unsuitable for sharing.
- **Decision**: Implemented a "Green Tree" (immutable, position-independent, stores width) and a "Red Node" (transient cursor, stores absolute offset).
- **Rationale**: This is the industry-standard approach (Roslyn, Rowan) for immutable syntax trees. It allows O(1) reuse of subtrees and O(log N) access to absolute positions.

### [2025-12-02] Conservative Incremental Re-lexing

- **Context**: When a user edits code, we want to re-lex as little as possible. However, re-lexing arbitrary ranges is dangerous due to context sensitivity (e.g., opening a comment).
- **Decision**: We only attempt incremental re-lexing if the edit is strictly contained within the *content* of a `Delimited` node (e.g., inside `{ ... }`). If the edit touches the delimiters themselves, we fail and fall back to a wider scope.
- **Rationale**: This guarantees that the surrounding context (the delimiters) remains invariant, making it safe to swap out the internals without re-analyzing the parent.
