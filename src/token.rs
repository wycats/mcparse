use crate::atom::AtomKind;
use crate::language::Delimiter;
use miette::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub span: SourceSpan,
}

impl SourceLocation {
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
}

#[derive(Debug, Clone)]
/// The recursive structure produced by the atomic lexer.
/// Can be a single token, a delimited group (which contains a list of TokenTrees), or a sequence group.
pub enum TokenTree {
    Token(Token),
    Delimited(Delimiter, Vec<TokenTree>, SourceLocation),
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
            TokenTree::Delimited(d, children, _) => {
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
