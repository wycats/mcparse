use mcparse::{
    atom::AtomKind,
    define_language,
    language::Delimiter,
    lexer::lex,
    shape::{MatchContext, MatchResult, Shape, choice, enter, opt, separated, seq, term},
    token::TokenStream,
};

// --- Language ---

define_language! {
    struct JsonPlusLang;
    atoms = [
        atom Whitespace = regex r"\s+",
        atom Operator = ":",
        atom Operator = ",",
        atom String = regex r#""([^"\\]|\\.)*""#,
        atom Number = regex r"\d+",
    ];
    delimiters = [
        delimiter "brace" = "{", "}",
        delimiter "bracket" = "[", "]",
    ];
}

// --- Shapes ---

#[derive(Clone, Copy, Debug)]
struct JsonValue;

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
            Delimiter {
                kind: "brace",
                open: "{",
                close: "}",
            },
            opt(separated(pair, term(","))),
        );

        let array = enter(
            Delimiter {
                kind: "bracket",
                open: "[",
                close: "]",
            },
            opt(separated(JsonValue, term(","))),
        );

        let shape = choice(string, choice(number, choice(object, array)));

        shape.match_shape(stream, context)
    }
}

fn main() {
    let lang = JsonPlusLang::new();
    let input = r#"{ "key": "value", "list": [ 1, 2, 3 ] }"#;

    println!("Input: {}", input);

    let trees = lex(input, &lang);
    let stream = TokenStream::new(&trees);

    // We don't use Parser here because we are matching a specific shape, not an expression with operators.
    // But we need a MatchContext.
    use mcparse::shape::NoOpMatchContext;
    let mut context = NoOpMatchContext;

    let result = JsonValue.match_shape(stream, &mut context);

    match result {
        Ok((tree, _)) => println!("Parsed: {:?}", tree),
        Err(_) => println!("Parse failed"),
    }
}
