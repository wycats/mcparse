use mcparse::{
    atom::{Atom, AtomKind, VariableRole},
    highlighter::{HighlightStyle, Highlighter},
    language::{Delimiter, Language, VariableRules},
    lexer::lex,
    r#macro::{ExpansionResult, Macro, MacroContext},
    shape::{Matcher, Shape, seq, term},
    token::{Cursor, SourceLocation, Token, TokenStream, TokenTree},
};

// --- Atoms ---

#[derive(Debug)]
struct Whitespace;
impl Atom for Whitespace {
    fn kind(&self) -> AtomKind {
        AtomKind::Whitespace
    }
    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)> {
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
    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        highlighter.highlight(token, HighlightStyle::None);
    }
}

#[derive(Debug)]
struct Identifier;
impl Atom for Identifier {
    fn kind(&self) -> AtomKind {
        AtomKind::Identifier(VariableRole::None)
    }
    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)> {
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
                    },
                    input.advance(len),
                ));
            }
        }
        None
    }
    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        highlighter.highlight(token, HighlightStyle::Variable);
    }
}

#[derive(Debug)]
struct Operator(String);
impl Atom for Operator {
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

#[derive(Debug)]
struct NumberLiteral;
impl Atom for NumberLiteral {
    fn kind(&self) -> AtomKind {
        AtomKind::Number
    }
    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)> {
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
    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        highlighter.highlight(token, HighlightStyle::Number);
    }
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

// --- Macros ---

#[derive(Debug)]
struct LetMacro {
    signature: Box<dyn Shape>,
}

impl LetMacro {
    fn new() -> Self {
        // let <ident> = <expr>
        // Signature matches arguments AFTER "let".
        let shape = seq(term(AnyIdentifier), seq(term("="), term(AtomKind::Number)));
        Self {
            signature: Box::new(shape),
        }
    }
}

impl Macro for LetMacro {
    fn name(&self) -> &str {
        "let"
    }
    fn signature(&self) -> &dyn Shape {
        &*self.signature
    }
    fn expand(
        &self,
        args: TokenTree,
        _lhs: Option<TokenTree>,
        _context: &MacroContext,
    ) -> ExpansionResult {
        ExpansionResult::Ok(args)
    }
}

// --- Language ---

#[derive(Debug)]
struct MiniScriptLang {
    atoms: Vec<Box<dyn Atom>>,
    delimiters: Vec<Delimiter>,
    macros: Vec<Box<dyn Macro>>,
    variable_rules: Box<dyn VariableRules>,
}

impl MiniScriptLang {
    fn new() -> Self {
        Self {
            atoms: vec![
                Box::new(Whitespace),
                Box::new(Operator("=".into())),
                Box::new(Operator(";".into())),
                Box::new(Identifier),
                Box::new(NumberLiteral),
            ],
            delimiters: vec![
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
            ],
            macros: vec![Box::new(LetMacro::new())],
            variable_rules: Box::new(MiniScriptVariableRules),
        }
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

impl Language for MiniScriptLang {
    fn atoms(&self) -> &[Box<dyn Atom>] {
        &self.atoms
    }
    fn delimiters(&self) -> &[Delimiter] {
        &self.delimiters
    }
    fn macros(&self) -> &[Box<dyn Macro>] {
        &self.macros
    }
    fn variable_rules(&self) -> &dyn VariableRules {
        self.variable_rules.as_ref()
    }
}

fn main() {
    let lang = MiniScriptLang::new();
    let input = "let x = 42";

    println!("Input: {}", input);

    let trees = lex(input, &lang);
    let stream = TokenStream::new(&trees);

    // Manually invoke the macro
    use mcparse::shape::NoOpMatchContext;
    let mut context = NoOpMatchContext;

    // We need to skip "let" to match the signature
    let args_stream = stream.advance(1); // Skip "let"
    // Wait, stream.advance(1) skips the first token.
    // The first token is "let" (Identifier).
    // But wait, "let" is an identifier in my atoms.
    // So yes, skip it.

    let let_macro = &lang.macros()[0];
    let signature = let_macro.signature();

    match signature.match_shape(args_stream, &mut context) {
        Ok((args, _)) => {
            println!("Matched Args: {:?}", args);
            let context = MacroContext;
            match let_macro.expand(args, None, &context) {
                ExpansionResult::Ok(expanded) => println!("Expanded: {:?}", expanded),
                ExpansionResult::Error(e) => println!("Expansion Error: {}", e),
            }
        }
        Err(_) => println!("Signature match failed"),
    }
}
