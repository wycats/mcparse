# Shapes & Structure

Now that we have our atoms, we can define the shape of JSON data.

JSON is recursive: an Object contains Values, and a Value can be an Object. In McParse, we define recursive grammars by implementing the `Shape` trait on a struct.

## The `Shape` Trait

The `Shape` trait has one method: `match_shape`. It takes a `TokenStream` and returns a `MatchResult`.

```rust
use mcparse::{
    shape::{Shape, MatchContext, MatchResult, term, seq, choice, enter, opt, separated},
    token::TokenStream,
    AtomKind,
    language::Delimiter,
};

#[derive(Clone, Copy, Debug)]
struct JsonValue;

impl Shape for JsonValue {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        // ... definition goes here ...
        # unimplemented!()
    }
}
```

## Building the Grammar

We can build the grammar using **Combinators**.

### 1. Primitive Values

Strings and Numbers are just "terms" (terminal symbols).

```rust
# use mcparse::{term, AtomKind};
let string = term(AtomKind::String);
let number = term(AtomKind::Number);
// let boolean = ...
// let null = ...
```

### 2. Arrays

An array is a bracketed list of values, separated by commas.
We use `enter` to go inside the brackets, and `separated` to match the list.

```rust
# use mcparse::{enter, opt, separated, term, Shape, MatchContext, MatchResult, token::TokenStream, language::Delimiter};
# #[derive(Clone, Copy, Debug)] struct JsonValue;
# impl Shape for JsonValue { fn match_shape<'a>(&self, s: TokenStream<'a>, c: &mut dyn MatchContext) -> MatchResult<'a> { unimplemented!() } }
let array = enter(
    Delimiter { kind: "bracket", open: "[", close: "]" },
    opt(separated(JsonValue, term(","))),
);
```

Note that we use `JsonValue` recursively here!

### 3. Objects

An object is a braced list of Key-Value pairs.
A Key-Value pair is a String, followed by a Colon, followed by a Value.

```rust
# use mcparse::{enter, opt, separated, term, seq, AtomKind, Shape, MatchContext, MatchResult, token::TokenStream, language::Delimiter};
# #[derive(Clone, Copy, Debug)] struct JsonValue;
# impl Shape for JsonValue { fn match_shape<'a>(&self, s: TokenStream<'a>, c: &mut dyn MatchContext) -> MatchResult<'a> { unimplemented!() } }
let pair = seq(term(AtomKind::String), seq(term(":"), JsonValue));

let object = enter(
    Delimiter { kind: "brace", open: "{", close: "}" },
    opt(separated(pair, term(","))),
);
```

### 4. Putting it all together

Finally, a JSON value is a choice between all these options.

```rust
# use mcparse::{shape::{Shape, MatchContext, MatchResult, term, seq, choice, enter, opt, separated}, token::TokenStream, AtomKind, language::Delimiter};
# #[derive(Clone, Copy, Debug)] struct JsonValue;
impl Shape for JsonValue {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        let string = term(AtomKind::String);
        let number = term(AtomKind::Number);

        let pair = seq(term(AtomKind::String), seq(term(":"), JsonValue));
        let object = enter(
            Delimiter { kind: "brace", open: "{", close: "}" },
            opt(separated(pair, term(","))),
        );

        let array = enter(
            Delimiter { kind: "bracket", open: "[", close: "]" },
            opt(separated(JsonValue, term(","))),
        );

        let shape = choice(string, choice(number, choice(object, array)));

        shape.match_shape(stream, context)
    }
}
```

## Using the Parser

Now you can parse some JSON!

```rust
# use mcparse::{shape::{Shape, MatchContext, MatchResult, term, seq, choice, enter, opt, separated}, token::TokenStream, AtomKind, language::{Delimiter, Language}, r#macro::Macro};
# #[derive(Clone, Copy, Debug)] struct JsonValue;
# impl Shape for JsonValue { fn match_shape<'a>(&self, s: TokenStream<'a>, c: &mut dyn MatchContext) -> MatchResult<'a> { Ok((mcparse::token::TokenTree::Empty, s)) } }
# #[derive(Debug)] struct JsonLang;
# impl JsonLang { fn new() -> Self { Self } }
# impl Language for JsonLang { fn macros(&self) -> &[Box<dyn Macro>] { &[] } fn binding_pass(&self) -> &dyn mcparse::scoping::BindingPass { &mcparse::scoping::NoOpBindingPass } fn reference_pass(&self) -> &dyn mcparse::scoping::ReferencePass { &mcparse::scoping::NoOpReferencePass } fn atoms(&self) -> &[Box<dyn mcparse::Atom>] { &[] } fn delimiters(&self) -> &[Delimiter] { &[] } }
fn main() {
    let lang = JsonLang::new();
    let input = r#"{ "key": "value", "list": [ 1, 2, 3 ] }"#;

    let trees = mcparse::lexer::lex(input, &lang);
    let stream = TokenStream::new(&trees);
    let mut context = mcparse::shape::NoOpMatchContext;

    let result = JsonValue.match_shape(stream, &mut context);

    match result {
        Ok((tree, _)) => println!("Parsed: {:?}", tree),
        Err(e) => println!("Parse failed: {:?}", e),
    }
}
```
