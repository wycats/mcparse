use crate::token::{Cursor, Token};
use crate::highlighter::Highlighter;
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableRole {
    None,
    Binding,
    Reference,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AtomKind {
    Whitespace,
    Identifier(VariableRole),
    Keyword(String),
    Literal,
    Operator,
    // Delimiters are handled separately
    Other(String),
}

pub trait Atom: Debug + Send + Sync {
    fn kind(&self) -> AtomKind;
    
    // We'll need a ParseResult type. For now, let's say Result<(Token, Cursor), ()>
    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)>;

    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter);
}
