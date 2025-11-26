use crate::highlighter::Highlighter;
use crate::token::{Cursor, Token};
use std::fmt::{self, Debug, Display};

/// The kind of an atomic token.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AtomKind {
    /// Whitespace (spaces, tabs, newlines). Usually ignored by shapes.
    Whitespace,
    /// An identifier (variable name, function name, etc.).
    Identifier,
    /// A string literal.
    String,
    /// A numeric literal.
    Number,
    /// An operator (e.g., `+`, `-`, `*`).
    Operator,
    /// Any other token kind not covered above.
    Other(String),
}

impl Display for AtomKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AtomKind::Whitespace => write!(f, "whitespace"),
            AtomKind::Identifier => write!(f, "identifier"),
            AtomKind::String => write!(f, "string"),
            AtomKind::Number => write!(f, "number"),
            AtomKind::Operator => write!(f, "operator"),
            AtomKind::Other(s) => write!(f, "{}", s),
        }
    }
}

/// A trait for defining atomic tokens.
///
/// Atoms are the building blocks of the lexer. They define how to parse a single token
/// from the input string and how to highlight it.
///
/// Prefer using the `define_atom!` macro to implement this trait.
pub trait Atom: Debug + Send + Sync {
    /// Returns the kind of token this atom produces.
    fn kind(&self) -> AtomKind;

    /// Tries to parse a token from the current cursor position.
    /// Returns `Some((Token, Cursor))` if successful, or `None` if the input doesn't match.
    fn parse<'a>(&self, input: Cursor<'a>) -> Option<(Token, Cursor<'a>)>;

    /// Applies syntax highlighting to the token.
    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter);
}
