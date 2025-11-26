use mcparse::{
    atom::{Atom, AtomKind, VariableRole},
    define_atom, define_language,
    highlighter::{HighlightStyle, Highlighter},
    language::{Delimiter, VariableRules},
    lexer::lex,
    shape::{MatchContext, MatchResult, Matcher, Shape, seq, term},
    token::{Cursor, SourceLocation, Token, TokenStream, TokenTree},
};
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

// --- Language Definition (Miniscript) ---

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
                    atom_index: None,
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
    }
    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)> {
        if input.rest.starts_with(&self.0) {
            Some((
                Token {
                    kind: AtomKind::Operator,
                    text: self.0.clone(),
                    location: SourceLocation {
                        span: (input.offset, self.0.len()).into(),
                    },
                    atom_index: None,
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
    struct Identifier;
    kind = AtomKind::Identifier(VariableRole::None);
    parse(input) {
        let mut chars = input.rest.chars();
        if let Some(c) = chars.next() {
            if c.is_alphabetic() || c == '_' {
                let mut len = c.len_utf8();
                for c in chars {
                    if c.is_alphanumeric() || c == '_' {
                        len += c.len_utf8();
                    } else {
                        break;
                    }
                }
                return Some((
                    Token {
                        kind: AtomKind::Identifier(VariableRole::None),
                        text: input.rest[..len].to_string(),
                        location: SourceLocation {
                            span: (input.offset, len).into(),
                        },
                        atom_index: None,
                    },
                    input.advance(len),
                ));
            }
        }
        None
    }
    highlight(token, highlighter) {
        highlighter.highlight(token, HighlightStyle::Variable);
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
                    atom_index: None,
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

#[derive(Debug)]
struct MiniScriptVariableRules;
impl VariableRules for MiniScriptVariableRules {
    fn classify(&self, prev: Option<&Token>, curr: &Token) -> VariableRole {
        if matches!(curr.kind, AtomKind::Identifier(_)) {
            if let Some(p) = prev {
                if p.text == "let" {
                    return VariableRole::Binding;
                }
            }
        }
        VariableRole::None
    }
}

define_language! {
    struct MiniScriptLang;
    atoms = [
        Whitespace,
        Punctuation("=".into()),
        Punctuation(";".into()),
        Punctuation("(".into()),
        Punctuation(")".into()),
        Punctuation("{".into()),
        Punctuation("}".into()),
        Identifier,
        NumberLiteral
    ];
    delimiters = [
        Delimiter {
            kind: "brace",
            open: "{",
            close: "}",
        },
        Delimiter {
            kind: "paren",
            open: "(",
            close: ")",
        },
    ];
    variable_rules = MiniScriptVariableRules;
}

// --- Matchers ---

#[derive(Debug, Clone)]
struct AnyIdentifier;
impl Matcher for AnyIdentifier {
    fn matches(&self, tree: &TokenTree) -> bool {
        match tree {
            TokenTree::Token(token) => matches!(token.kind, AtomKind::Identifier(_)),
            _ => false,
        }
    }

    fn describe(&self) -> String {
        "Identifier".to_string()
    }
}

// --- Shape ---

#[derive(Clone, Copy, Debug)]
struct MiniScriptShape;

impl Shape for MiniScriptShape {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        // let <ident> = <number>
        let let_stmt = seq(
            term("let"),
            seq(term(AnyIdentifier), seq(term("="), term(AtomKind::Number))),
        );

        // For now, just match one statement
        let_stmt.match_shape(stream, context)
    }
}

// --- Error Reporting ---

#[derive(Error, Debug, Diagnostic)]
#[error("Parse Error")]
struct ParseError {
    #[source_code]
    src: NamedSource<String>,

    #[label("Here")]
    span: SourceSpan,

    #[help]
    advice: String,
}

fn main() -> miette::Result<()> {
    let lang = MiniScriptLang::new();

    // Example 1: Unknown token error (Lexer level)
    let input1 = "let x = 123 @";
    println!("\n--- Example 1: Lexer Error ---");
    if let Err(e) = report_errors(input1, &lang) {
        println!("{:?}", e);
    } else {
        println!("Parsed successfully!");
    }

    // Example 2: Parse error (Shape level)
    // "let" expects identifier, then "=", then number.
    // Here we give it a number where identifier is expected.
    let input2 = "let 123 = 456";
    println!("\n--- Example 2: Parse Error (Wrong Token Type) ---");
    if let Err(e) = report_errors(input2, &lang) {
        println!("{:?}", e);
    } else {
        println!("Parsed successfully!");
    }

    // Example 3: Missing token
    let input3 = "let x 123";
    println!("\n--- Example 3: Parse Error (Missing Token) ---");
    if let Err(e) = report_errors(input3, &lang) {
        println!("{:?}", e);
    } else {
        println!("Parsed successfully!");
    }

    Ok(())
}

fn report_errors(input: &str, lang: &impl mcparse::language::Language) -> miette::Result<()> {
    let trees = lex(input, lang);
    let stream = TokenStream::new(&trees);

    // 1. Check for Lexer Errors (Unknown tokens)
    for tree in &trees {
        if let TokenTree::Token(token) = tree {
            if let AtomKind::Other(ref s) = token.kind {
                if s == "Unknown" {
                    return Err(ParseError {
                        src: NamedSource::new("input", input.to_string()),
                        span: token.location.span,
                        advice: format!("Unexpected character: '{}'", token.text),
                    }
                    .into());
                }
            }
        }
    }

    // 2. Check for Parser Errors
    use mcparse::shape::NoOpMatchContext;
    let mut context = NoOpMatchContext;

    match MiniScriptShape.match_shape(stream, &mut context) {
        Ok(_) => {
            println!("Parsed successfully!");
            Ok(())
        }
        Err(e) => Err(ParseError {
            src: NamedSource::new("input", input.to_string()),
            span: e.span,
            advice: e.message,
        }
        .into()),
    }
}
