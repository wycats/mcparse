# Contextual Keywords

> **Perspective**: In a modern, evolving language, the concept of a "Reserved Word" is an anti-pattern. Ideally, all keywords should be contextual.

McParse takes this philosophy to heart. In fact, **McParse has no built-in concept of a Reserved Word**.

When you define `keywords` in `define_language!`, you are creating atoms that match specific strings and highlight them as keywords. However, these atoms produce `AtomKind::Identifier` tokens.

This means that to the parser, a keyword is just an identifier with specific text.

## The "Reserved Word" Problem

In traditional parsers, if you add `async` to your keyword list, it becomes a distinct token type (`ASYNC`). A parser rule expecting an `IDENTIFIER` will fail if it encounters `ASYNC`. This breaks any user code that used `async` as a variable name.

## The McParse Solution

In McParse, since keywords are just identifiers:

1.  `term("async")` matches an identifier with the text "async".
2.  `term(AtomKind::Identifier)` matches _any_ identifier, **including "async"**.

This means you can introduce new keywords without breaking existing variable usage, provided your grammar is unambiguous.

### Example: `async`

Suppose you want to add `async` as a keyword.

1.  **Highlighting**: Add `"async"` to your `keywords` list in `define_language!`. This ensures it gets syntax highlighting.
2.  **Parsing**: Use `term("async")` in your grammar where you expect the keyword.
3.  **Compatibility**: Existing code like `let async = 5;` continues to work!
    - The lexer produces `async` as an `Identifier` token (highlighted as a keyword).
    - The `let` grammar expects an `Identifier`.
    - `async` matches `Identifier`.
    - Success!

## Variable Binding and Scoping

Since `AtomKind::Keyword` does not exist, determining which identifiers are variable bindings and which are references requires analyzing the token stream.

In McParse, this is handled by the `BindingPass` and `ReferencePass` traits, which run **after** lexing but **before** macro expansion.

> **Important**: Variable binding happens on the `TokenTree` structure. This allows for more complex rules than simple lookbehind, but it still happens before parsing the full grammar.

This architecture allows you to implement complex scoping rules (like block scoping, shadowing, and hoisting) without complicating the core lexer.
