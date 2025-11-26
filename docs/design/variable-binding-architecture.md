# Variable Binding & Scoping Architecture

## Overview

This document outlines a revised architecture for handling variable bindings and references in McParse. The goal is to decouple variable identification from the initial lexical analysis while ensuring it happens _before_ macro expansion. This allows for more robust scoping rules without complicating the core atomic lexer.

## Retrospective: The Atomic Lexing Oversight

### The Initial Mistake

In earlier iterations of McParse, we attempted to handle variable binding (hygiene) purely at the **Atomic Lexing** phase. The `Language` trait had a `VariableRules` method that allowed looking at the _previous token_ to decide if the current identifier was a binding or a reference.

### Why It Failed

This approach was fundamentally flawed for several reasons:

1.  **Conflation of Concerns**: It tried to mix "what is this token?" (Lexing) with "what does this token mean?" (Semantics).
2.  **Insufficient Context**: A simple lookbehind (`prev_token`) is not enough to handle complex grammars. For example, in `let (a, b) = ...`, `a` and `b` are bindings, but neither immediately follows `let`.
3.  **Scope Ignorance**: The lexer produces a flat stream (or a tree during construction), but it doesn't inherently track _scope boundaries_. To correctly resolve references, you need a stack of scopes, which requires traversing the structure _after_ it is built.
4.  **Shadowing & Hoisting**: Complex scoping rules like shadowing (inner `let x` hides outer `let x`) or hoisting (function declarations visible before definition) are impossible to implement in a single-pass lexer without building significant side-state that mirrors parsing.

### The Correction

We moved to a **Multi-Pass Architecture**:

1.  **Lexing**: Produces a structural `TokenTree`.
2.  **Binding Pass**: Traverses the tree to identify declarations and build the scope graph.
3.  **Reference Pass**: Traverses the tree to resolve usages against the scope graph.
4.  **Parsing**: Matches shapes against the fully resolved tree.

This separation allows the lexer to remain simple and fast, while the scoping passes can be as complex as the language requires (e.g., implementing block scoping, function scoping, or module scoping) without complicating the grammar or the lexer.

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
