# Project Plan Outline

## Phase 1: Design & Prototyping (Current)

- [x] Refine `DESIGN.md` with concrete Rust interfaces.
- [x] Create an aspirational example of a language definition.
- [x] Prototype the core `Atom` and `Shape` traits in Rust.
- [x] Implement a basic `TokenStream` and `AtomicLexer`.
- [x] Implement a basic syntax highlighter and printer for verification.
- [x] Integrate `miette` for source location verification.

## Phase 2: Core Parsing Engine

- [ ] Implement Shape Algebra (Sequence, Choice, Repetition).
- [ ] Implement Adjacency constraints and whitespace handling.
- [ ] Implement basic Macro expansion loop.
- [ ] Add integration tests for core parsing logic.

## Phase 3: Tooling Support

- [ ] Implement Syntax Highlighting infrastructure.
- [ ] Implement Tab Completion infrastructure.
- [ ] Implement Source Location tracking and Incremental updates.

## Phase 4: Advanced Features

- [ ] Variable Binding and Hygiene.
- [ ] Error Recovery and Forgiving Parsing.
- [ ] Wasm bindings and TypeScript integration.
