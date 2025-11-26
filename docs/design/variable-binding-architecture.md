# Variable Binding & Scoping Architecture

## Overview

This document outlines a revised architecture for handling variable bindings and references in McParse. The goal is to decouple variable identification from the initial lexical analysis while ensuring it happens _before_ macro expansion. This allows for more robust scoping rules without complicating the core atomic lexer.

## The 4-Step Process

1.  **Flat Lexical Scan**:

    - Input: Raw text.
    - Action: Use the language's `Atom` definitions to produce a stream of atoms.
    - State: No keywords, bindings, or references yet—just identifiers and literals.

2.  **Token Tree Construction**:

    - Input: Stream of atoms.
    - Action: Apply delimiter rules to form a recursive `TokenTree`.
    - Result: A structured tree of tokens.

3.  **Variable Binding & Reference Resolution**:

    - Input: `TokenTree`.
    - Action: The language converts specific tokens (typically identifiers) into `Binding` and `Reference` tokens.
    - **Key Concept**: This is where scoping happens. Since we have a `TokenTree`, we can scope identifiers to their containing blocks (delimiters).
    - State: Tokens are now classified as Bindings, References, or plain Identifiers.

4.  **Macro Expansion**:
    - Input: `TokenTree` (with Bindings/References).
    - Action: Expand macros.
    - Constraint: Macros cannot expand into syntax that creates _new_ bindings or references. The binding structure is fixed before expansion.

## Conceptual Model

- **Variable Pass as Macro Expansion**: The variable resolution step can be thought of as a specialized, initial macro expansion pass. It transforms a `TokenTree` into another `TokenTree`, but strictly for the purpose of classifying bindings and references.
- **Context-Free Syntax**: To support this, the syntax defining bindings must be context-free regarding macro expansion. Macros cannot hide or generate binding declarations that weren't visible in the unexpanded tree.

## Implementation Strategy

The language should provide a convenience mechanism that operates in two sub-passes:

1.  **Binding Assignment**:

    - Scan the tree to identify tokens that act as bindings.
    - Assign them a unique symbol identifier.
    - Associate them with an enclosing scope (a specific delimiter).

2.  **Reference Resolution**:
    - Scan the tree for identifier tokens within the scope of established bindings.
    - Convert matching tokens into `Reference` tokens pointing to the unique symbol identifier of the binding.

This approach avoids hardcoding "identifier" and allows languages to define what tokens serve as variables, while providing a standard engine for block-based scoping.
