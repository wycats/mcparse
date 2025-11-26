# Implementation Plan - Phase 5: API Refinement & Ergonomics

This phase focuses on refining the API surface to be more intuitive and "magical" while maintaining full composability. We want to bridge the gap between "easy" regex-like definitions and "hard" custom implementations without creating a cliff.

## Goals

1.  **Declarative Atom Syntax**: Implement a way to define Atoms using regex or simple rules (e.g., for whitespace, identifiers) that compiles down to the standard `Atom` trait.
2.  **Seamless Composition**: Ensure that declarative atoms can be mixed freely with custom `Atom` implementations.
3.  **Reduce Boilerplate**: Simplify the `Language` trait implementation for common cases.
4.  **Documentation Update**: Add a "Custom Implementation" tutorial to the book to show *why* and *how* to drop down to the lower level.

## Proposed Steps

1.  **Design**: Refine the `define_language!` macro design in `docs/design/ergonomics.md` to support mixed declarative/custom atoms.
2.  **Implementation**:
    - Add `regex` dependency (if not present) or use a lightweight alternative.
    - Implement a `RegexAtom` struct that implements `Atom`.
    - Create/Update `define_language!` macro to support `atom Name = r"..."` syntax.
3.  **Verification**: Refactor `JsonPlus` example to use the new declarative syntax where appropriate, keeping custom logic where necessary.
4.  **Documentation**:
    - Update the "JSON Parser" tutorial to use the new syntax.
    - Create a new "Advanced: Custom Atoms" tutorial explaining the manual implementation.
5.  **Contextual Keywords**: Document how to add keywords backwards-compatibly using custom `VariableRules`.
