# Deferred Work

## Variable Binding & Scoping Architecture (Phase 6)

- Implement the 4-step process: Flat Lex -> Token Tree -> Binding/Ref Pass -> Macro Expansion.
- Refactor `VariableRules` to be a post-lexing pass on `TokenTree`.
- Implement scoping logic to associate bindings with enclosing delimiters.
- Ensure macro expansion respects the fixed binding structure.
