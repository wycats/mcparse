use crate::atom::AtomKind;
use crate::language::Delimiter;
use miette::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub span: SourceSpan,
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
    Empty,
}

impl TokenTree {
    pub fn empty() -> Self {
        TokenTree::Empty
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
