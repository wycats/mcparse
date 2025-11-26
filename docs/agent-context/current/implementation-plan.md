# Phase 6: Variable Binding & Scoping Architecture

## Overview

We are moving to a 4-step parsing process:

1. Flat Lexical Scan (Atoms only)
2. Token Tree Construction (Delimiters)
3. Variable Binding & Reference Resolution (Scoping)
4. Macro Expansion

This phase focuses on implementing Step 3 and ensuring Step 4 respects it.

## Steps

1.  **Design Traits**: Define `BindingPass` and `ReferencePass` traits that languages can implement to define their scoping rules.
2.  **Refactor Lexer**: Ensure the initial lexer produces "raw" identifiers without binding information.
3.  **Implement Scoping Engine**: Create a default implementation that walks the `TokenTree`, tracks scopes (entered via delimiters), and resolves references.
4.  **Update Macros**: Modify the macro expander to expect fully resolved bindings and references.
5.  **Migration**: Update existing examples (`miniscript`) to use the new system.
