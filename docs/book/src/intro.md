# Introduction & Philosophy

McParse is a new kind of parsing toolkit designed for the modern era of IDEs and Language Servers.

## Why McParse?

Traditional parsing tools (Regex, Lex/Yacc, Parser Combinators) were designed for **batch processing**: read a file, parse it, output an AST or an error. If the input is invalid, they stop. If you change one character, they start over.

McParse is designed for **interactive environments**:

- **Resilient**: It can parse incomplete or invalid code without crashing, preserving as much structure as possible. This is critical for IDEs, where the code is "broken" 99% of the time while the user is typing.
- **Incremental**: It can re-parse just the parts that changed. This enables real-time feedback even for large files.
- **Introspective**: It provides rich information for syntax highlighting, autocompletion, and refactoring out of the box.
- **Macro-Aware**: It treats macros as first-class citizens, allowing the language syntax to be extended dynamically while maintaining hygiene and tooling support.

## Core Concepts

McParse is built on two foundational pillars: **Atomic Lexing** and **Shape Algebra**.

### Atomic Lexing

Most parsers treat the input as a stream of flat tokens (e.g., `IF`, `LPAREN`, `ID`, `RPAREN`). McParse takes a different approach.

The **Atomic Lexer** performs a first pass that handles:

1.  **Delimiters**: Parentheses `()`, braces `{}`, and brackets `[]` are matched during lexing. The output is not a flat stream, but a tree of `TokenTree`s. This ensures that the parser never sees unbalanced delimiters.
2.  **Atoms**: Basic units like strings, numbers, and identifiers are classified.
3.  **Variable Classification**: Identifiers are tentatively classified as bindings or references based on local context (e.g., "after `let`"), enabling "syntax-directed" hygiene.

This "lexical structure" is robust and rarely changes, even if the high-level grammar is broken.

### Shape Algebra

Once we have a tree of atoms, we use **Shape Algebra** to define the grammar.

Shapes are combinators (like `seq`, `choice`, `rep`) that match patterns in the `TokenTree`. Unlike traditional parser combinators, Shapes:

- **Operate on Trees**: They can `enter` a delimited group (like a block `{ ... }`) and parse its contents.
- **Handle Whitespace**: They can enforce adjacency constraints (e.g., `obj.method` vs `obj . method`) while generally ignoring whitespace.
- **Recover from Errors**: They have built-in strategies to skip over bad tokens and resynchronize, ensuring the rest of the file is parsed correctly.

## The "McParse Way"

1.  **Parsing is for Tools**: We prioritize the needs of the IDE (highlighting, completion, go-to-definition) over the needs of the compiler (generating machine code).
2.  **Errors are Normal**: A syntax error is just another kind of node in the tree. The parser should never panic or abort.
3.  **Hygiene is Structural**: Variable binding and scoping should be handled by the parser structure, not by a separate semantic analysis pass. This allows macros to be hygienic by default.
