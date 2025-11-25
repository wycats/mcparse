# Project Plan Outline

## Phase 1: Design & Prototyping (Completed)

- [x] Refine `DESIGN.md` with concrete Rust interfaces.
- [x] Create an aspirational example of a language definition.
- [x] Prototype the core `Atom` and `Shape` traits in Rust.
- [x] Implement a basic `TokenStream` and `AtomicLexer`.
- [x] Implement a basic syntax highlighter and printer for verification.
- [x] Integrate `miette` for source location verification.

## Phase 2: Core Parsing Engine & Semantics (Completed)

- [x] Implement Shape Algebra (Sequence, Choice, Repetition).
- [x] Implement Adjacency constraints and whitespace handling.
- [x] Implement Macro expansion loop with Variable Binding/Hygiene.
- [x] Implement Error Recovery strategies in Shapes.
- [x] Add integration tests for core parsing logic.

## Phase 3: Tooling & Incrementalism (Completed)

- [x] Implement a pretty printer for `TokenTree` (better `Debug` format).
- [x] Add `macro_rules!` macros to reduce boilerplate in Shape/Atom definitions.
- [x] Implement advanced Syntax Highlighting (semantic tokens).
- [x] Create interactive demos (TUI/REPL):
  - Real-time syntax highlighting.
  - Interactive tab completion (Deferred).
- [x] Design Incremental Parsing strategy.
- [ ] Implement Incremental Parsing and re-lexing (Moved to Phase 5).

## Phase 4: Documentation & Guides

- [ ] Set up `mdbook` for the project.
- [ ] Write "The McParse Book":
  - Introduction & Philosophy.
  - Tutorial: Building a JSON Parser.
  - Tutorial: Building a Scripting Language.
  - Advanced Topics: Macros, Hygiene, Error Recovery.
- [ ] Generate API documentation (`cargo doc`).

## Phase 5: Advanced Features & Ecosystem

- [ ] Implement Incremental Parsing (Red/Green Trees).
- [ ] Declarative Macro Syntax.
- [ ] Gradual Typing Syntax exploration.
- [ ] Wasm bindings and TypeScript integration.
