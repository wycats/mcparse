# Implementation Plan: Design & Prototyping

## Goal

Solidify the design of McParse by defining concrete Rust interfaces and validating them against an aspirational example language.

## Steps

1.  **Design Review**: Analyze `DESIGN.md` for potential issues with the "Atomic Lexing + Shape Constraints" approach.
2.  **Interface Sketching**: Draft Rust traits for:
    - `Atom` / `AtomKind`
    - `Shape`
    - `Highlighter`
    - `MacroSignature`
3.  **Example Language**: Define a "Toy Language" (e.g., a JSON-like or simple expression language) using the sketched interfaces to see if the ergonomics feel right.
4.  **Refinement**: Update `DESIGN.md` with the refined interfaces.
