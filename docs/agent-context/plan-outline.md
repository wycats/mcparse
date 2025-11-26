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
  - Interactive tab completion.
- [x] Design Incremental Parsing strategy.
- [x] Implement Incremental Parsing and re-lexing (Moved to Phase 5).

## Phase 4: Documentation & Guides (Completed)

- [x] Set up `mdbook` for the project.
- [x] Write "The McParse Book":
  - Introduction & Philosophy.
  - Tutorial: Building a JSON Parser.
  - Tutorial: Building a Scripting Language.
  - Advanced Topics: Macros, Hygiene, Error Recovery.
- [x] Generate API documentation (`cargo doc`).

## Phase 5: API Refinement & Ergonomics (Completed)

- [x] Review API surface based on documentation experience.
- [x] Design declarative syntax for Atoms (e.g., regex-like but constraint-aware).
- [x] Simplify boilerplate for common patterns (Whitespace, Identifiers).
- [x] Evaluate `define_atom!` and `define_language!` macros for improvements.

## Phase 6: Variable Binding & Scoping Architecture (Completed)

- [x] Design the `BindingPass` and `ReferencePass` traits.
- [x] Refactor `VariableRules` to operate on `TokenTree` (post-lexing).
- [x] Implement block-based scoping logic.
- [x] Update Macro Expansion to respect pre-calculated bindings.
- [x] Verify context-free syntax constraints.

## Phase 7: Incremental Parsing Implementation (Completed)

- [x] Implement Red/Green Tree data structures.
- [x] Implement Tree-Based Invalidation logic.
- [x] Implement Conservative Re-lexing.

## Phase 8: Documentation (Completed)

- [x] Update "The McParse Book" with Incremental Parsing guide.
- [x] Review and update Reference documentation (Hygiene, Macros).

## Phase 9: Scoping & Completion (Completed)

- [x] Refactor `VariableRules` to be a post-lexing pass.
- [x] Implement intelligent Tab Completion for variables using `ScopeStack`.
- [x] Enhance `VariableRules` with context (delimiter stack).

## Phase 10: DSL Refinement

- [ ] Implement `define_language!` as a procedural macro.
- [ ] Implement declarative atom syntax (regex-like).

## Phase 11: Incremental Scoping & Semantics

- [ ] Design incremental binding/reference resolution.
- [ ] Implement "Binding Invalidation" logic (track dirty scopes).
- [ ] Integrate Macro Expansion into the incremental pipeline.

## Phase 12: Advanced Features & Ecosystem

- [ ] Wasm bindings and TypeScript integration.
- [ ] Gradual Typing Syntax exploration.
