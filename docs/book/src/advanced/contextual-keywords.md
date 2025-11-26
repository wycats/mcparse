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

## Variable Rules and Hygiene

Since `AtomKind::Keyword` does not exist, your `VariableRules` must rely on token text to decide when a binding occurs.

> **Important**: Variable binding happens during the **Lexing** phase, _before_ macro expansion. This means macros cannot dynamically introduce bindings based on their expansion. The binding status of an identifier is fixed once the lexer runs.

The default `PatternVariableRules` handles this automatically if you configure it correctly.

```rust
use mcparse::language::PatternVariableRules;

let rules = PatternVariableRules::new()
    .bind_after_keyword("let")
    .bind_after_keyword("fn");
```

If you implement `VariableRules` manually, check the text of the previous token:

```rust
use mcparse::{
    token::Token,
    atom::{AtomKind, VariableRole},
    language::VariableRules,
};

#[derive(Debug)]
struct MyVariableRules;

impl VariableRules for MyVariableRules {
    fn classify(&self, prev: Option<&Token>, curr: &Token) -> VariableRole {
        // Only classify identifiers
        if !matches!(curr.kind, AtomKind::Identifier(_)) {
            return VariableRole::None;
        }

        if let Some(prev) = prev {
            // Check text instead of AtomKind::Keyword
            match prev.text.as_str() {
                "let" | "fn" => return VariableRole::Binding,
                "alias" => return VariableRole::Binding, // Contextual keyword
                _ => {}
            }
        }

        VariableRole::Reference
    }
}
```

### A Note on Scope

`PatternVariableRules` uses a simple lookbehind heuristic. It does not understand blocks, scopes, or nesting. It purely checks if the _immediately preceding token_ matches a keyword.

For example, in:

```rust
let x = 1;
{
    let x = 2;
}
```

Both `x`s are identified as bindings because they both follow `let`.

However, in:

```rust
let (a, b) = (1, 2);
```

`PatternVariableRules` will _not_ identify `a` and `b` as bindings because they do not immediately follow `let`. For complex patterns, you may need a more sophisticated `VariableRules` implementation or rely on the parser to refine the binding information later (though McParse prefers to do it at lexing time for highlighting).
