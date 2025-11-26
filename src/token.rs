use crate::atom::AtomKind;
use crate::language::Delimiter;
use miette::SourceSpan;

/// A unique identifier for a variable binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BindingId(pub usize);

/// Represents a location in the source code.
/// Wraps `miette::SourceSpan` to provide location tracking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub span: SourceSpan,
}

impl SourceLocation {
    pub fn new(start: usize, len: usize) -> Self {
        Self {
            span: (start, len).into(),
        }
    }

    /// Checks if the given offset is contained within this location.
    pub fn contains(&self, offset: usize) -> bool {
        let start = self.span.offset();
        let end = start + self.span.len();
        offset >= start && offset <= end
    }
}

#[derive(Debug, Clone)]
/// A single atomic unit of code (identifier, keyword, operator, etc.).
pub struct Token {
    pub kind: AtomKind,
    pub text: String,
    pub location: SourceLocation,
    /// The index of the atom in the language definition that produced this token.
    /// Used for syntax highlighting.
    pub atom_index: Option<usize>,
    /// The ID of the variable binding this token refers to or defines.
    pub binding: Option<BindingId>,
}

impl Token {
    pub fn new(kind: AtomKind, text: &str, offset: usize) -> Self {
        Self {
            kind,
            text: text.to_string(),
            location: SourceLocation::new(offset, text.len()),
            atom_index: None,
            binding: None,
        }
    }
}

#[derive(Debug, Clone)]
/// The recursive structure produced by the atomic lexer.
/// Can be a single token, a delimited group (which contains a list of TokenTrees), or a sequence group.
pub enum TokenTree {
    Token(Token),
    Delimited(Delimiter, Vec<TokenTree>, SourceLocation, bool),
    Group(Vec<TokenTree>), // For sequences
    Error(String),
    Empty,
}

impl TokenTree {
    pub fn empty() -> Self {
        TokenTree::Empty
    }

    pub fn to_sexp(&self) -> String {
        match self {
            TokenTree::Token(t) => format!("{:?}", t.text),
            TokenTree::Delimited(d, children, _, _) => {
                let inner: Vec<String> = children.iter().map(|c| c.to_sexp()).collect();
                format!("({} {})", d.kind, inner.join(" "))
            }
            TokenTree::Group(children) => {
                let inner: Vec<String> = children.iter().map(|c| c.to_sexp()).collect();
                format!("(group {})", inner.join(" "))
            }
            TokenTree::Error(msg) => format!("(error {:?})", msg),
            TokenTree::Empty => "(empty)".to_string(),
        }
    }
}

/// A cursor pointing to a specific position in the input string.
/// Used by the lexer to track progress.
#[derive(Debug, Clone, Copy)]
pub struct Cursor<'a> {
    pub rest: &'a str,
    pub offset: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            rest: input,
            offset: 0,
        }
    }

    pub fn advance(&self, amt: usize) -> Self {
        Self {
            rest: &self.rest[amt..],
            offset: self.offset + amt,
        }
    }
}

/// A stream of `TokenTree`s.
/// This is the input to the parser and shapes.
/// It is a lightweight slice over the token trees.
#[derive(Debug, Clone)]
pub struct TokenStream<'a> {
    pub trees: &'a [TokenTree],
}

impl<'a> TokenStream<'a> {
    pub fn new(trees: &'a [TokenTree]) -> Self {
        Self { trees }
    }

    pub fn is_empty(&self) -> bool {
        self.trees.is_empty()
    }

    pub fn first(&self) -> Option<&TokenTree> {
        self.trees.first()
    }

    pub fn advance(&self, n: usize) -> Self {
        Self {
            trees: &self.trees[n..],
        }
    }
}
