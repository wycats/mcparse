# Defining Atoms

In McParse, the first step is to define the "Atoms" of your language. These are the raw tokens that the lexer will identify.

For JSON, we need:

1.  **Whitespace**: Spaces, tabs, newlines.
2.  **Strings**: `"hello"`.
3.  **Numbers**: `123`, `3.14`.
4.  **Booleans**: `true`, `false`.
5.  **Null**: `null`.
6.  **Punctuation**: `:`, `,`.

## The `define_language!` Macro

McParse provides a powerful macro to define your language's atoms using Regular Expressions and Keywords.

```rust
use mcparse::{define_language, language::Delimiter};

define_language! {
    struct JsonLang;
    atoms = [
        // Whitespace
        atom Whitespace = regex r"\s+",

        // Strings
        // Matches a double quote, followed by any number of non-escaped characters or escaped characters, followed by a double quote.
        atom String = regex r#""([^"\\]|\\.)*""#,

        // Numbers
        // Matches optional minus, integer part, optional fraction, optional exponent.
        atom Number = regex r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?",

        // Keywords
        keywords [ "true", "false", "null" ],

        // Punctuation
        atom Operator = ":",
        atom Operator = ",",
    ];
    delimiters = [
        delimiter "brace" = "{", "}",
        delimiter "bracket" = "[", "]",
    ];
}
```

## Delimiters

Delimiters (like `{}` and `[]`) are special. The lexer matches them and creates a recursive `TokenTree` structure. You don't define them as Atoms; you define them in the `delimiters` section of `define_language!`.

In the example above, we defined braces `{}` and brackets `[]` as delimiters. This means that when the lexer encounters a `{`, it will recursively parse tokens until it finds a matching `}`.

## Custom Atoms

For more complex parsing logic that cannot be expressed easily with Regex (e.g. nested comments, complex string interpolation), you can implement the `Atom` trait manually. See the [Custom Atoms](../../advanced/custom-atoms.md) chapter for details.

With this, we have a working lexer! In the next section, we'll use these atoms to define the grammar shapes.
