use crate::atom::{Atom, AtomKind, VariableRole};
use crate::highlighter::{HighlightStyle, Highlighter};
use crate::language::{Delimiter, Language, PatternVariableRules, VariableRules};
use crate::r#macro::Macro;
use crate::token::{Cursor, SourceLocation, Token};

#[derive(Debug)]
pub struct WhitespaceAtom;

impl Atom for WhitespaceAtom {
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
            let text = input.rest[..len].to_string();
            let token = Token {
                kind: AtomKind::Whitespace,
                text,
                location: SourceLocation {
                    span: (input.offset, len).into(),
                },
                atom_index: None,
            };
            Some((token, input.advance(len)))
        } else {
            None
        }
    }

    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        highlighter.highlight(token, HighlightStyle::None);
    }
}

#[derive(Debug)]
pub struct IdentifierAtom;

impl Atom for IdentifierAtom {
    fn kind(&self) -> AtomKind {
        AtomKind::Identifier(VariableRole::None)
    }

    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)> {
        let mut len = 0;
        for (i, c) in input.rest.chars().enumerate() {
            if i == 0 {
                if c.is_alphabetic() || c == '_' {
                    len += c.len_utf8();
                } else {
                    return None;
                }
            } else {
                if c.is_alphanumeric() || c == '_' {
                    len += c.len_utf8();
                } else {
                    break;
                }
            }
        }

        if len > 0 {
            let text = input.rest[..len].to_string();
            let token = Token {
                kind: AtomKind::Identifier(VariableRole::None),
                text,
                location: SourceLocation {
                    span: (input.offset, len).into(),
                },
                atom_index: None,
            };
            Some((token, input.advance(len)))
        } else {
            None
        }
    }

    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        highlighter.highlight(token, HighlightStyle::Variable);
    }
}

#[derive(Debug)]
pub struct KeywordAtom {
    keywords: Vec<String>,
}

impl KeywordAtom {
    pub fn new(keywords: &[&str]) -> Self {
        Self {
            keywords: keywords.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Atom for KeywordAtom {
    fn kind(&self) -> AtomKind {
        AtomKind::Identifier(VariableRole::None)
    }

    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)> {
        for kw in &self.keywords {
            if input.rest.starts_with(kw) {
                // Check boundary
                let next_char = input.rest[kw.len()..].chars().next();
                if next_char.map_or(true, |c| !c.is_alphanumeric() && c != '_') {
                    let token = Token {
                        kind: AtomKind::Identifier(VariableRole::None),
                        text: kw.clone(),
                        location: SourceLocation {
                            span: (input.offset, kw.len()).into(),
                        },
                        atom_index: None,
                    };
                    return Some((token, input.advance(kw.len())));
                }
            }
        }
        None
    }

    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        highlighter.highlight(token, HighlightStyle::Keyword);
    }
}

#[derive(Debug)]
pub struct SymbolAtom {
    symbols: Vec<String>,
}

impl SymbolAtom {
    pub fn new(symbols: &[&str]) -> Self {
        Self {
            symbols: symbols.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Atom for SymbolAtom {
    fn kind(&self) -> AtomKind {
        AtomKind::Operator
    }

    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)> {
        for sym in &self.symbols {
            if input.rest.starts_with(sym) {
                let token = Token {
                    kind: AtomKind::Operator,
                    text: sym.clone(),
                    location: SourceLocation {
                        span: (input.offset, sym.len()).into(),
                    },
                    atom_index: None,
                };
                return Some((token, input.advance(sym.len())));
            }
        }
        None
    }

    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        highlighter.highlight(token, HighlightStyle::Operator);
    }
}

#[derive(Debug)]
pub struct MockLanguage {
    atoms: Vec<Box<dyn Atom>>,
    delimiters: Vec<Delimiter>,
    macros: Vec<Box<dyn Macro>>,
    variable_rules: Box<dyn VariableRules>,
}

impl MockLanguage {
    pub fn new() -> Self {
        Self {
            atoms: vec![
                Box::new(WhitespaceAtom),
                Box::new(KeywordAtom::new(&["let"])),
                Box::new(IdentifierAtom),
            ],
            delimiters: vec![Delimiter {
                kind: "paren",
                open: "(",
                close: ")",
            }],
            macros: vec![],
            variable_rules: Box::new(PatternVariableRules::new()),
        }
    }

    pub fn with_keyword_binding(mut self, keyword: &str) -> Self {
        // This is a bit hacky because we boxed the rules.
        // Ideally we'd build the rules first.
        // For now, let's just replace it.
        let mut rules = PatternVariableRules::new();
        rules = rules.bind_after_keyword(keyword);
        self.variable_rules = Box::new(rules);
        self
    }

    pub fn with_macro(mut self, mac: Box<dyn Macro>) -> Self {
        self.macros.push(mac);
        self
    }

    pub fn with_symbol(mut self, symbol: &str) -> Self {
        self.atoms.insert(1, Box::new(SymbolAtom::new(&[symbol]))); // Insert before identifier
        self
    }
}

impl Language for MockLanguage {
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
