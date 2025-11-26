use crate::atom::{Atom, AtomKind, VariableRole};
use crate::highlighter::{HighlightStyle, Highlighter};
use crate::token::{Cursor, Token};
use regex::Regex;
use std::fmt::Debug;

/// An Atom implementation that uses a regular expression to match tokens.
///
/// This is useful for defining tokens like whitespace, identifiers, and literals
/// without writing a custom state machine.
pub struct RegexAtom {
    kind: AtomKind,
    regex: Regex,
}

impl Debug for RegexAtom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegexAtom")
            .field("kind", &self.kind)
            .field("regex", &self.regex.as_str())
            .finish()
    }
}

impl RegexAtom {
    /// Creates a new RegexAtom.
    ///
    /// The pattern is automatically anchored to the start of the string if it isn't already.
    pub fn new(kind: AtomKind, pattern: &str) -> Self {
        let pattern = if pattern.starts_with('^') {
            pattern.to_string()
        } else {
            format!("^{}", pattern)
        };

        Self {
            kind,
            regex: Regex::new(&pattern).expect("Invalid regex pattern"),
        }
    }
}

impl Atom for RegexAtom {
    fn kind(&self) -> AtomKind {
        self.kind.clone()
    }

    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)> {
        if let Some(mat) = self.regex.find(input.rest) {
            let len = mat.end();
            if len == 0 {
                return None;
            }

            Some((
                Token::new(self.kind.clone(), &input.rest[..len], input.offset),
                input.advance(len),
            ))
        } else {
            None
        }
    }

    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        let style = match self.kind {
            AtomKind::String => HighlightStyle::String,
            AtomKind::Number => HighlightStyle::Number,
            AtomKind::Operator => HighlightStyle::Operator,
            AtomKind::Whitespace => HighlightStyle::None,
            AtomKind::Identifier(_) => HighlightStyle::Variable,
            _ => HighlightStyle::None,
        };
        highlighter.highlight(token, style);
    }
}

/// An Atom implementation that matches a fixed set of keywords.
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
        // Find the longest matching keyword to avoid prefix issues (e.g. matching "in" inside "int")

        let mut best_match: Option<&String> = None;

        for keyword in &self.keywords {
            if input.rest.starts_with(keyword) {
                if let Some(current_best) = best_match {
                    if keyword.len() > current_best.len() {
                        best_match = Some(keyword);
                    }
                } else {
                    best_match = Some(keyword);
                }
            }
        }

        if let Some(keyword) = best_match {
            let len = keyword.len();
            Some((
                Token::new(
                    AtomKind::Identifier(VariableRole::None),
                    keyword,
                    input.offset,
                ),
                input.advance(len),
            ))
        } else {
            None
        }
    }

    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        highlighter.highlight(token, HighlightStyle::Keyword);
    }
}

/// An Atom implementation that matches a specific literal string.
#[derive(Debug)]
pub struct LiteralAtom {
    kind: AtomKind,
    literal: String,
}

impl LiteralAtom {
    pub fn new(kind: AtomKind, literal: &str) -> Self {
        Self {
            kind,
            literal: literal.to_string(),
        }
    }
}

impl Atom for LiteralAtom {
    fn kind(&self) -> AtomKind {
        self.kind.clone()
    }

    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)> {
        if input.rest.starts_with(&self.literal) {
            let len = self.literal.len();
            Some((
                Token::new(self.kind.clone(), &self.literal, input.offset),
                input.advance(len),
            ))
        } else {
            None
        }
    }

    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter) {
        let style = match self.kind {
            AtomKind::Operator => HighlightStyle::Operator,
            _ => HighlightStyle::None,
        };
        highlighter.highlight(token, style);
    }
}
