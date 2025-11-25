# Project Plan Outline

## Phase 1: Design & Prototyping (Completed)

- [x] Refine `DESIGN.md` with concrete Rust interfaces.
- [x] Create an aspirational example of a language definition.
- [x] Prototype the core `Atom` and `Shape` traits in Rust.
- [x] Implement a basic `TokenStream` and `AtomicLexer`.
- [x] Implement a basic syntax highlighter and printer for verification.
- [x] Integrate `miette` for source location verification.

## Phase 2: Core Parsing Engine & Semantics (Active)

- [ ] Implement Shape Algebra (Sequence, Choice, Repetition).
- [ ] Implement Adjacency constraints and whitespace handling.
- [ ] Implement Macro expansion loop with Variable Binding/Hygiene.
- [ ] Implement Error Recovery strategies in Shapes.
- [ ] Add integration tests for core parsing logic.

## Phase 3: Tooling & Incrementalism

- [ ] Implement advanced Syntax Highlighting (semantic tokens).
- [ ] Implement Tab Completion infrastructure.
- [ ] Implement Incremental Parsing and re-lexing.

## Phase 4: Advanced Features & Ecosystem

- [ ] Declarative Macro Syntax.
- [ ] Gradual Typing Syntax exploration.
- [ ] Wasm bindings and TypeScript integration.
