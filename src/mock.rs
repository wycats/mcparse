use crate::atom::{Atom, AtomKind};
use crate::token::{Cursor, Token, SourceLocation};
use crate::highlighter::{Highlighter, HighlightStyle};

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
        AtomKind::Identifier
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
                kind: AtomKind::Identifier,
                text,
                location: SourceLocation {
                    span: (input.offset, len).into(),
                },
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
