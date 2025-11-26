use crate::atom::AtomKind;
use crate::language::Delimiter;
use crate::token::{Token, TokenStream, TokenTree};
use miette::{Diagnostic, SourceSpan};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error, Diagnostic)]
#[error("{message}")]
pub struct ParseError {
    #[label("here")]
    pub span: SourceSpan,
    pub message: String,
}

impl ParseError {
    pub fn new(span: SourceSpan, message: String) -> Self {
        Self { span, message }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionKind {
    Keyword,
    Variable,
    Function,
    Field,
    Operator,
    Type,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
    pub delete_backwards: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum AdjacencyConstraint {
    None,
    Required,
    Forbidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    Left,
    Right,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Precedence(pub u32);

pub type MatchResult<'a> = Result<(TokenTree, TokenStream<'a>), ParseError>;

/// Context provided to shapes during matching.
/// Allows shapes to perform complex logic like expression parsing with precedence.
pub trait MatchContext {
    fn parse_expression<'a>(
        &mut self,
        stream: TokenStream<'a>,
        precedence: Precedence,
    ) -> MatchResult<'a>;
}

pub struct NoOpMatchContext;
impl MatchContext for NoOpMatchContext {
    fn parse_expression<'a>(
        &mut self,
        stream: TokenStream<'a>,
        _precedence: Precedence,
    ) -> MatchResult<'a> {
        // Default implementation fails
        let span = if let Some(TokenTree::Token(t)) = stream.first() {
            t.location.span
        } else {
            (0, 0).into()
        };
        Err(ParseError::new(
            span,
            "Expression parsing not supported".into(),
        ))
    }
}

/// The core trait for defining the grammar.
/// A Shape consumes tokens from a `TokenStream` and produces a `TokenTree`.
pub trait Shape: Debug + Send + Sync {
    /// Tries to match the shape against the token stream.
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
    ) -> MatchResult<'a>;

    /// Returns the adjacency constraint for this shape.
    fn adjacency(&self) -> AdjacencyConstraint {
        AdjacencyConstraint::None
    }

    /// Provides completion items for the given cursor position.
    fn complete<'a>(
        &self,
        _stream: TokenStream<'a>,
        _context: &mut dyn MatchContext,
        _cursor: usize,
    ) -> Vec<CompletionItem> {
        vec![]
    }
}

// Matcher Trait
/// Defines how to match a single `TokenTree`.
/// Implementations exist for `AtomKind` (match by kind), `&str` (match by text), and `Delimiter` (match by delimiter type).
pub trait Matcher: Debug + Send + Sync {
    fn matches(&self, tree: &TokenTree) -> bool;
    fn describe(&self) -> String;
    fn suggest(&self, _current_token: &Token) -> Vec<CompletionItem> {
        vec![]
    }
    fn suggest_insertion(&self) -> Vec<CompletionItem> {
        vec![]
    }
}

impl Matcher for AtomKind {
    fn matches(&self, tree: &TokenTree) -> bool {
        match tree {
            TokenTree::Token(token) => token.kind == *self,
            _ => false,
        }
    }
    fn describe(&self) -> String {
        self.to_string()
    }
    // AtomKind doesn't know specific values, so it can't suggest much without context.
    // But we could suggest "an identifier" or "a number" as a placeholder?
    // For now, empty.
}

impl Matcher for &str {
    fn matches(&self, tree: &TokenTree) -> bool {
        match tree {
            TokenTree::Token(token) => token.text == *self,
            _ => false,
        }
    }

    fn describe(&self) -> String {
        format!("'{}'", self)
    }

    fn suggest(&self, current_token: &Token) -> Vec<CompletionItem> {
        if self.starts_with(&current_token.text) {
            vec![CompletionItem {
                label: self.to_string(),
                kind: CompletionKind::Keyword,
                detail: None,
                delete_backwards: current_token.text.len(),
            }]
        } else {
            vec![]
        }
    }

    fn suggest_insertion(&self) -> Vec<CompletionItem> {
        vec![CompletionItem {
            label: self.to_string(),
            kind: CompletionKind::Keyword,
            detail: None,
            delete_backwards: 0,
        }]
    }
}

impl Matcher for Delimiter {
    fn matches(&self, tree: &TokenTree) -> bool {
        match tree {
            TokenTree::Delimited(d, _, _, _) => d.kind == self.kind,
            _ => false,
        }
    }

    fn describe(&self) -> String {
        format!("Delimiter '{}'", self.kind)
    }
}

// Primitives

// term
/// Matches a single token or tree using the provided `Matcher`.
/// Implicitly skips leading whitespace.
#[derive(Debug, Clone)]
pub struct Term<M: Matcher>(pub M);

impl<M: Matcher> Shape for Term<M> {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        _context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        let mut current_stream = stream;

        // Skip whitespace
        while let Some(tree) = current_stream.first() {
            if let TokenTree::Token(token) = tree
                && token.kind == AtomKind::Whitespace
            {
                current_stream = current_stream.advance(1);
                continue;
            }
            break;
        }

        if let Some(tree) = current_stream.first() {
            if self.0.matches(tree) {
                return Ok((tree.clone(), current_stream.advance(1)));
            }

            let (span, found) = match tree {
                TokenTree::Token(t) => (t.location.span, t.kind.to_string()),
                TokenTree::Delimited(d, _, loc, _) => (loc.span, format!("Delimiter {}", d.kind)),
                TokenTree::Group(_) => ((0, 0).into(), "Group".to_string()),
                TokenTree::Error(_) => ((0, 0).into(), "Error".to_string()),
                TokenTree::Empty => ((0, 0).into(), "Empty".to_string()),
            };

            return Err(ParseError::new(
                span,
                format!("Expected {}, found {}", self.0.describe(), found),
            ));
        }

        Err(ParseError::new(
            (0, 0).into(),
            format!("Expected {}, found EOF", self.0.describe()),
        ))
    }

    fn complete<'a>(
        &self,
        stream: TokenStream<'a>,
        _context: &mut dyn MatchContext,
        cursor: usize,
    ) -> Vec<CompletionItem> {
        let mut current_stream = stream;

        // Skip whitespace
        while let Some(tree) = current_stream.first() {
            if let TokenTree::Token(token) = tree
                && token.kind == AtomKind::Whitespace
            {
                if token.location.contains(cursor) {
                    return self.0.suggest_insertion();
                }
                current_stream = current_stream.advance(1);
                continue;
            }
            break;
        }

        if let Some(tree) = current_stream.first() {
            match tree {
                TokenTree::Token(token) => {
                    if token.location.contains(cursor) {
                        return self.0.suggest(token);
                    }
                    if token.location.span.offset() >= cursor {
                        return self.0.suggest_insertion();
                    }
                }
                TokenTree::Delimited(_, _, loc, _) => {
                    if loc.span.offset() >= cursor {
                        return self.0.suggest_insertion();
                    }
                }
                _ => {}
            }
        } else {
            return self.0.suggest_insertion();
        }

        vec![]
    }
}

pub fn term<M: Matcher>(matcher: M) -> Term<M> {
    Term(matcher)
}

// seq
/// Matches shape `A` followed by shape `B`.
/// Implicitly skips whitespace between `A` and `B` (because `B`'s first term will skip it).
#[derive(Debug, Clone)]
pub struct Seq<A, B>(pub A, pub B);

impl<A: Shape, B: Shape> Shape for Seq<A, B> {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        let (res_a, stream_after_a) = self.0.match_shape(stream, context)?;
        let (res_b, stream_after_b) = self.1.match_shape(stream_after_a, context)?;
        Ok((TokenTree::Group(vec![res_a, res_b]), stream_after_b))
    }

    fn complete<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
        cursor: usize,
    ) -> Vec<CompletionItem> {
        // Try to match A
        match self.0.match_shape(stream.clone(), context) {
            Ok((_, stream_after_a)) => {
                // If A matched, check if cursor is inside A's consumed range?
                // Actually, match_shape doesn't return the range easily.
                // But if A matched, we can try to complete B.
                // However, we also need to check if we are *still* inside A.

                // Heuristic: If A produces completions, return them.
                // If not, try B.
                let completions_a = self.0.complete(stream, context, cursor);
                if !completions_a.is_empty() {
                    return completions_a;
                }

                self.1.complete(stream_after_a, context, cursor)
            }
            Err(_) => {
                // A failed, so we are likely in A
                self.0.complete(stream, context, cursor)
            }
        }
    }
}

pub fn seq<A: Shape, B: Shape>(a: A, b: B) -> Seq<A, B> {
    Seq(a, b)
}

// choice
/// Ordered choice: tries to match `A`, if it fails, tries to match `B`.
#[derive(Debug, Clone)]
pub struct Choice<A, B>(pub A, pub B);

impl<A: Shape, B: Shape> Shape for Choice<A, B> {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        match self.0.match_shape(stream.clone(), context) {
            Ok(res) => Ok(res),
            Err(_) => self.1.match_shape(stream, context),
        }
    }

    fn complete<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
        cursor: usize,
    ) -> Vec<CompletionItem> {
        let mut items = self.0.complete(stream.clone(), context, cursor);
        items.extend(self.1.complete(stream, context, cursor));
        items
    }
}

pub fn choice<A: Shape, B: Shape>(a: A, b: B) -> Choice<A, B> {
    Choice(a, b)
}

// rep
/// Matches shape `A` zero or more times.
#[derive(Debug, Clone)]
pub struct Rep<A>(pub A);

impl<A: Shape> Shape for Rep<A> {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        let mut current_stream = stream;
        let mut results = Vec::new();

        while let Ok((res, next_stream)) = self.0.match_shape(current_stream.clone(), context) {
            if next_stream.trees.len() == current_stream.trees.len() {
                // Matched empty, break to avoid infinite loop
                results.push(res);
                break;
            }

            results.push(res);
            current_stream = next_stream;
        }

        Ok((TokenTree::Group(results), current_stream))
    }

    fn complete<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
        cursor: usize,
    ) -> Vec<CompletionItem> {
        let mut current_stream = stream;

        loop {
            // Check if we can complete in the current position
            let items = self.0.complete(current_stream.clone(), context, cursor);
            if !items.is_empty() {
                return items;
            }

            // If not, try to advance
            match self.0.match_shape(current_stream.clone(), context) {
                Ok((_, next_stream)) => {
                    if next_stream.trees.len() == current_stream.trees.len() {
                        break;
                    }
                    current_stream = next_stream;
                }
                Err(_) => break,
            }
        }
        vec![]
    }
}

pub fn rep<A: Shape>(a: A) -> Rep<A> {
    Rep(a)
}

// enter
/// Matches a delimited group (e.g., `(...)`) and then matches the `inner` shape against the content of that group.
/// Implicitly skips leading whitespace before the group.
#[derive(Debug, Clone)]
pub struct Enter<S>(pub Delimiter, pub S);

impl<S: Shape> Shape for Enter<S> {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        // 1. Match delimiter (skipping whitespace)
        let mut current_stream = stream;

        while let Some(tree) = current_stream.first() {
            if let TokenTree::Token(token) = tree
                && token.kind == AtomKind::Whitespace
            {
                current_stream = current_stream.advance(1);
                continue;
            }
            break;
        }

        if let Some(TokenTree::Delimited(d, content, loc, _)) = current_stream.first()
            && d.kind == self.0.kind
        {
            // 2. Create new stream from content
            let inner_stream = TokenStream::new(content);

            // 3. Match inner
            let (res, remaining_inner) = self.1.match_shape(inner_stream, context)?;

            // 4. Ensure inner consumed everything (Implicit Exit/End)
            let mut check_stream = remaining_inner;
            while let Some(tree) = check_stream.first() {
                if let TokenTree::Token(token) = tree
                    && token.kind == AtomKind::Whitespace
                {
                    check_stream = check_stream.advance(1);
                    continue;
                }
                // Found non-whitespace, so inner didn't consume everything
                let span = match tree {
                    TokenTree::Token(t) => t.location.span,
                    TokenTree::Delimited(_, _, loc, _) => loc.span,
                    _ => loc.span, // Fallback
                };
                return Err(ParseError::new(span, "Expected end of group".into()));
            }

            return Ok((res, current_stream.advance(1)));
        }

        let span = if let Some(TokenTree::Token(t)) = current_stream.first() {
            t.location.span
        } else {
            (0, 0).into()
        };
        Err(ParseError::new(
            span,
            format!("Expected {}", self.0.describe()),
        ))
    }

    fn complete<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
        cursor: usize,
    ) -> Vec<CompletionItem> {
        let mut current_stream = stream;

        // Skip whitespace
        while let Some(tree) = current_stream.first() {
            if let TokenTree::Token(token) = tree
                && token.kind == AtomKind::Whitespace
            {
                current_stream = current_stream.advance(1);
                continue;
            }
            break;
        }

        if let Some(TokenTree::Delimited(d, content, loc, _)) = current_stream.first()
            && d.kind == self.0.kind
            && loc.contains(cursor)
        {
            let inner_stream = TokenStream::new(content);
            return self.1.complete(inner_stream, context, cursor);
        }
        vec![]
    }
}

pub fn enter<S: Shape>(delimiter: Delimiter, inner: S) -> Enter<S> {
    Enter(delimiter, inner)
}

// adjacent
/// Matches shape `A` followed by shape `B` with **no** intervening whitespace.
/// Used for tight binding (e.g., `obj.prop`).
#[derive(Debug, Clone)]
pub struct Adjacent<A, B>(pub A, pub B);

impl<A: Shape, B: Shape> Shape for Adjacent<A, B> {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        let (res_a, stream_after_a) = self.0.match_shape(stream, context)?;

        // Check for whitespace at the start of stream_after_a
        if let Some(TokenTree::Token(token)) = stream_after_a.first()
            && token.kind == AtomKind::Whitespace
        {
            return Err(ParseError::new(
                token.location.span,
                "Unexpected whitespace".into(),
            ));
        }

        let (res_b, stream_after_b) = self.1.match_shape(stream_after_a, context)?;
        Ok((TokenTree::Group(vec![res_a, res_b]), stream_after_b))
    }
}

pub fn adjacent<A: Shape, B: Shape>(a: A, b: B) -> Adjacent<A, B> {
    Adjacent(a, b)
}

// empty
/// Matches nothing and consumes no tokens. Always succeeds.
#[derive(Debug, Clone)]
pub struct Empty;

impl Shape for Empty {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        _context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        Ok((TokenTree::Empty, stream))
    }
}

pub fn empty() -> Empty {
    Empty
}

// end
/// Matches the end of the input stream. Fails if there are any remaining tokens.
#[derive(Debug, Clone)]
pub struct End;

impl Shape for End {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        _context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        let mut current_stream = stream;
        while let Some(tree) = current_stream.first() {
            if let TokenTree::Token(token) = tree
                && token.kind == AtomKind::Whitespace
            {
                current_stream = current_stream.advance(1);
                continue;
            }
            let span = match tree {
                TokenTree::Token(t) => t.location.span,
                TokenTree::Delimited(_, _, loc, _) => loc.span,
                _ => (0, 0).into(),
            };
            return Err(ParseError::new(span, "Expected end of input".into()));
        }
        Ok((TokenTree::Empty, current_stream))
    }
}

pub fn end() -> End {
    End
}

// expr
/// Matches an expression using the configured precedence rules and macros.
#[derive(Debug, Clone)]
pub struct Expr(pub Precedence);

impl Shape for Expr {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        context.parse_expression(stream, self.0)
    }
}

pub fn expr(precedence: Precedence) -> Expr {
    Expr(precedence)
}

// Derived

/// Matches `A` optionally. Equivalent to `choice(a, empty())`.
pub fn opt<A: Shape + Clone>(a: A) -> Choice<A, Empty> {
    choice(a, empty())
}

/// Matches `item` separated by `sep`.
/// e.g., `separated(term(Number), term(","))` matches `1, 2, 3`.
pub fn separated<A: Shape + Clone, S: Shape + Clone>(item: A, sep: S) -> Seq<A, Rep<Seq<S, A>>> {
    // seq(item, rep(seq(sep, item)))
    seq(item.clone(), rep(seq(sep, item)))
}

/// Matches `A` joined by adjacency (no whitespace).
pub fn joined<A: Shape + Clone>(a: A) -> Seq<A, Rep<Adjacent<Empty, A>>> {
    // seq(a, rep(adjacent(empty(), a)))
    seq(a.clone(), rep(adjacent(empty(), a)))
}

// recover
/// Tries to match `S`. If it fails, skips tokens until `M` matches (or EOF),
/// and returns a `TokenTree::Error`.
#[derive(Debug, Clone)]
pub struct Recover<S, M>(pub S, pub M);

impl<S: Shape, M: Matcher> Shape for Recover<S, M> {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
    ) -> MatchResult<'a> {
        match self.0.match_shape(stream.clone(), context) {
            Ok(res) => Ok(res),
            Err(_) => {
                let mut current_stream = stream;
                let mut skipped_count = 0;

                while let Some(tree) = current_stream.first() {
                    if self.1.matches(tree) {
                        break;
                    }
                    // Also stop if we hit a closing delimiter?
                    // For now, just rely on the matcher.

                    current_stream = current_stream.advance(1);
                    skipped_count += 1;
                }

                if skipped_count > 0 {
                    Ok((
                        TokenTree::Error(format!("Parse error, skipped {} tokens", skipped_count)),
                        current_stream,
                    ))
                } else {
                    // If we didn't skip anything and still failed (and didn't match terminator immediately),
                    // it means we are at EOF or terminator.
                    // If we are at terminator, we return Error but don't consume terminator.
                    Ok((TokenTree::Error("Parse error".to_string()), current_stream))
                }
            }
        }
    }
}

pub fn recover<S: Shape, M: Matcher>(shape: S, terminator: M) -> Recover<S, M> {
    Recover(shape, terminator)
}
