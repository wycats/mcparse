use mcparse::{
    atom::{Atom, AtomKind, VariableRole},
    define_atom, define_language,
    highlighter::{HighlightStyle, Highlighter},
    language::{Delimiter, Language, VariableRules},
    lexer::lex,
    r#macro::Macro,
    shape::{MatchContext, MatchResult, Shape, choice, enter, opt, separated, seq, term},
    token::{Cursor, SourceLocation, Token, TokenStream},
};

// --- Atoms ---

define_atom! {
    struct Whitespace;
    kind = AtomKind::Whitespace;
    parse(input) {
        let mut len = 0;
        for c in input.rest.chars() {
            if c.is_whitespace() {
                len += c.len_utf8();
            } else {
                break;
            }
        }
        if len > 0 {
            Some((
                Token {
                    kind: AtomKind::Whitespace,
                    text: input.rest[..len].to_string(),
                    location: SourceLocation {
                        span: (input.offset, len).into(),
                    },
                },
                input.advance(len),
            ))
        } else {
            None
        }
    }
    highlight(token, highlighter) {
        highlighter.highlight(token, HighlightStyle::None);
    }
}

#[derive(Debug)]
struct Punctuation(String);
impl Atom for Punctuation {
    fn kind(&self) -> AtomKind {
        AtomKind::Operator
    } // Reuse Operator for punctuation
    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)> {
        if input.rest.starts_with(&self.0) {
            Some((
                Token {
                    kind: AtomKind::Operator,
                    text: self.0.clone(),
                    location: SourceLocation {
                        span: (input.offset, self.0.len()).into(),
                    },
                },
                input.advance(self.0.len()),
            ))
        } else {
            None
        }
    }
    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        highlighter.highlight(token, HighlightStyle::Operator);
    }
}

define_atom! {
    struct StringLiteral;
    kind = AtomKind::String;
    parse(input) {
        if input.rest.starts_with('"') {
            let mut len = 1;
            let mut escaped = false;
            for c in input.rest[1..].chars() {
                len += c.len_utf8();
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    return Some((
                        Token {
                            kind: AtomKind::String,
                            text: input.rest[..len].to_string(),
                            location: SourceLocation {
                                span: (input.offset, len).into(),
                            },
                        },
                        input.advance(len),
                    ));
                }
            }
        }
        None
    }
    highlight(token, highlighter) {
        highlighter.highlight(token, HighlightStyle::String);
    }
}

define_atom! {
    struct NumberLiteral;
    kind = AtomKind::Number;
    parse(input) {
        let mut len = 0;
        for c in input.rest.chars() {
            if c.is_ascii_digit() {
                len += c.len_utf8();
            } else {
                break;
            }
        }
        if len > 0 {
            Some((
                Token {
                    kind: AtomKind::Number,
                    text: input.rest[..len].to_string(),
                    location: SourceLocation {
                        span: (input.offset, len).into(),
                    },
                },
                input.advance(len),
            ))
        } else {
            None
        }
    }
    highlight(token, highlighter) {
        highlighter.highlight(token, HighlightStyle::Number);
    }
}

// --- Language ---

#[derive(Debug)]
struct NoOpVariableRules;
impl VariableRules for NoOpVariableRules {
    fn classify(&self, _prev: Option<&Token>, _curr: &Token) -> VariableRole {
        VariableRole::None
    }
}

define_language! {
    struct JsonPlusLang;
    atoms = [
        Whitespace,
        Punctuation(":".into()),
        Punctuation(",".into()),
        StringLiteral,
        NumberLiteral
    ];
    delimiters = [
        Delimiter {
            kind: "brace",
            open: "{",
            close: "}",
        },
        Delimiter {
            kind: "bracket",
            open: "[",
            close: "]",
        },
    ];
    variable_rules = NoOpVariableRules;
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
