# Quick Start: A Number List

Let's get your hands dirty. We will build a parser for a simple comma-separated list of numbers: `1, 2, 3`.

## 1. Setup

Create a new Rust project and add `mcparse` and `miette` (for error reporting).

```bash
cargo new number_list
cd number_list
cargo add mcparse miette
```

## 2. Define Language

We can define our language, including its atoms, in one go using the `define_language!` macro.

```rust
use mcparse::{define_language, language::Delimiter};

define_language! {
    struct ListLang;
    atoms = [
        atom Whitespace = regex r"\s+",
        atom Number = regex r"\d+",
        atom Operator = ",",
    ];
    delimiters = [];
}
```

## 3. Parse!

Now use the `separated` shape to parse the list. We'll also use `miette` to print nice errors if something goes wrong.

```rust
# use mcparse::{define_language, language::{Delimiter, Language}, atom::{Atom, AtomKind as HiddenAtomKind}, r#macro::Macro};
# define_language! { struct ListLang; atoms = [ atom Whitespace = regex r"\s+", atom Number = regex r"\d+", atom Operator = ",", ]; delimiters = []; }
use mcparse::{lexer::lex, token::TokenStream, shape::{separated, term, Shape, NoOpMatchContext}, AtomKind};
use miette::{NamedSource, Report};

fn main() {
    // Let's try a broken input: missing number between commas!
    let input = "1, , 3";
    let lang = ListLang::new();

    // 1. Lex
    let trees = lex(input, &lang);
    let stream = TokenStream::new(&trees);

    // 2. Define Shape: Number separated by Comma
    // Note: We use term(",") to match the literal comma atom we defined.
    let list_shape = separated(term(AtomKind::Number), term(","));

    // 3. Match
    let mut context = NoOpMatchContext;
    let result = list_shape.match_shape(stream, &mut context);

    match result {
        Ok((tree, _)) => println!("Success: {:?}", tree),
        Err(e) => {
            // McParse errors implement miette::Diagnostic!
            let report = Report::new(e)
                .with_source_code(NamedSource::new("input.txt", input.to_string()));
            println!("{:?}", report);
        }
    }
}
```

If you run this, you'll see a beautiful error message pointing exactly to where the number was expected:

```text
Error:   × Expected number, found Delimiter ,
   ╭─[input.txt:1:4]
 1 │ 1, , 3
   ·    ─ here
   ╰────
```

Congratulations! You've parsed your first input (and handled your first error) with McParse.
