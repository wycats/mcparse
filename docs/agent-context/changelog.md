# Changelog

## Phase 1: Design & Prototyping (Completed)

- **Design**: Solidified `DESIGN.md` with "Expression Continuation" model and "Atomic Whitespace" handling.
- **Design**: Defined core Rust traits: `Atom`, `Shape`, `Macro`, `Language`.
- **Docs**: Added `docs/design/aspirational-language.md` with a JsonPlus example.
- **Implementation**: Created `examples/demo.rs` with a working lexer, highlighter, and `miette` error reporting.
- Initialized project structure and agent context.

## Phase 2: Core Parsing Engine & Semantics (Completed)

- **Shape Algebra**: Implemented core combinators (`term`, `seq`, `choice`, `rep`, `enter`, `adjacent`, `empty`, `end`) and derived combinators (`opt`, `separated`, `joined`) in `src/shape.rs`.
- **Atomic Lexer**: Implemented recursive descent lexer in `src/lexer.rs` producing `TokenTree`s.
- **Variable Binding**: Implemented `VariableRules` trait and integrated it into the lexer to classify identifiers as Bindings or References.
- **Macro Expansion**: Implemented `Parser` struct with `parse_expression` loop handling prefix and infix macros with precedence.
- **Error Recovery**: Implemented `recover` combinator and `TokenTree::Error` for robust parsing.
- **Integration Tests**:
  - Implemented `JsonPlus` example language demonstrating recursive shapes and delimited groups.
  - Implemented `MiniScript` example language demonstrating macros and variable binding.
- **Refactor**: Converted Shape constructors to return concrete types to support `Clone`.

## Phase 3: Tooling & Incrementalism (Completed)

- **Interactive REPL**: Built a TUI-based REPL (`examples/repl.rs`) using `ratatui` with real-time syntax highlighting and cursor tracking.
- **Structured Errors**: Refactored `Shape` trait to return `Result<T, ParseError>` and added `Matcher::describe()` for self-documenting parsers.
- **Error Reporting**: Created `examples/miniscript_errors.rs` demonstrating rich error diagnostics with `miette`.
- **Incremental Design**: Completed the design for Incremental Parsing (`docs/design/incremental-parsing.md`) using Red/Green trees.
- **Deferred**: Implementation of Incremental Parsing moved to Phase 5 to prioritize Documentation.

## Phase 4: Documentation & Guides (Completed)

- **The McParse Book**: Created a comprehensive guide using `mdbook` covering:
  - Core Concepts (Pipeline, Shape Algebra).
  - Tutorials (JSON Parser, Scripting Language).
  - Advanced Topics (Error Recovery, Incrementalism).
  - Cookbook & Reference.
- **Doc Testing**: Implemented a robust strategy to test book examples via `src/lib.rs` doctests.
- **API Docs**: Polished Rustdoc comments for the public API (`Shape`, `Atom`, `Token`).
- **Developer Experience**:
  - Integrated `mermaid.js` for diagrams.
  - Improved error reporting with `miette`.
  - Configured `mdbook` for local development (disabled broken Playground link).

## Phase 5: API Refinement & Ergonomics (Completed)

- **Declarative Atoms**: Implemented `RegexAtom`, `KeywordAtom`, and `LiteralAtom` to simplify language definitions.
- **Macro Syntax**: Updated `define_language!` to support a concise, declarative syntax for atoms and keywords (`atom Name = r"..."`, `keyword "if"`).
- **Documentation**:
  - Added "Custom Atoms" chapter explaining the escape hatch for manual atom implementation.
  - Updated "Contextual Keywords" chapter to clarify the interaction between atomic lexing and macro expansion.
  - Explicitly documented the "Lexing-Time Binding" constraint.
- **Examples**: Refactored `json_plus.rs` and `miniscript.rs` to use the new ergonomic syntax, reducing boilerplate significantly.
- **Verification**: Verified all book examples and unit tests pass with the new API.

## Phase 6: Variable Binding & Scoping (Completed)

- **Architecture**: Moved from "Lexing-Time Binding" to a multi-pass architecture (Lex -> Binding Pass -> Reference Pass -> Parse) to support block scoping and shadowing.
- **Core Traits**: Defined `BindingPass` and `ReferencePass` traits in `src/scoping.rs` for identifying and resolving variables.
- **Implementation**:
  - Implemented `ScopeStack` for managing nested scopes.
  - Implemented `SimpleBindingPass` and `SimpleReferencePass` as default implementations.
  - Updated `Token` to carry `binding: Option<BindingId>`.
- **Verification**: Created `examples/scoping_demo.rs` verifying correct handling of shadowing, block scopes, and hoisting checks.
- **Documentation**: Updated "The McParse Book" to reflect the new architecture and updated `docs/design/variable-binding-architecture.md`.
- **Migration**: Updated `miniscript` and `repl` examples to use the new API.
