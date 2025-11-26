# Phase 10: DSL Refinement (Outline)

## Goal

Refine the Domain Specific Language (DSL) for defining languages in McParse to be more ergonomic and powerful.

## Proposed Work

1.  **Procedural Macro for `define_language!`**:

    - Investigate if `macro_rules!` limitations warrant a move to procedural macros.
    - If so, implement `define_language!` as a proc-macro to allow for more flexible syntax and better error messages.
    - If not, refine the existing `macro_rules!` to support more features (e.g., better validation).

2.  **Declarative Atom Syntax**:

    - Enhance the declarative syntax for atoms (e.g., `atom Name = r"..."`) to support more complex patterns or constraints if needed.
    - Consider adding support for "composed" atoms or shared regex patterns.

3.  **Documentation**:
    - Update the "Building a Language" guide to reflect the refined DSL.
    - Add examples of advanced DSL usage.
