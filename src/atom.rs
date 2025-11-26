use crate::highlighter::Highlighter;
use crate::token::{Cursor, Token};
use std::fmt::{self, Debug, Display};

/// The role of a variable identifier, determined during lexing.
/// This allows for accurate syntax highlighting even before full parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableRole {
    /// The role is unknown or not applicable.
    None,
    /// The identifier is declaring a new variable (e.g., `let x`).
    Binding,
    /// The identifier is referring to an existing variable (e.g., `x + 1`).
    Reference,
}

/// The kind of an atomic token.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AtomKind {
    /// Whitespace (spaces, tabs, newlines). Usually ignored by shapes.
    Whitespace,
    /// An identifier (variable name, function name, etc.).
    Identifier(VariableRole),
    /// A language keyword (e.g., `if`, `else`, `fn`).
    Keyword(String),
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
            AtomKind::Identifier(role) => match role {
                VariableRole::None => write!(f, "identifier"),
                VariableRole::Binding => write!(f, "variable binding"),
                VariableRole::Reference => write!(f, "variable reference"),
            },
            AtomKind::Keyword(k) => write!(f, "keyword '{}'", k),
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
