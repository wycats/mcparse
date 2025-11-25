use crate::atom::AtomKind;
use crate::language::Delimiter;
use crate::token::{SourceLocation, TokenStream, TokenTree};
use std::fmt::Debug;

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

pub type MatchResult<'a> = Result<(TokenTree, TokenStream<'a>), ()>;

// Placeholder for CompletionFuture
pub type CompletionFuture<'a> = ();

pub trait MatchContext {
    fn parse_expression<'a>(&mut self, stream: TokenStream<'a>, precedence: Precedence) -> MatchResult<'a>;
}

pub struct NoOpMatchContext;
impl MatchContext for NoOpMatchContext {
    fn parse_expression<'a>(&mut self, _stream: TokenStream<'a>, _precedence: Precedence) -> MatchResult<'a> {
        Err(())
    }
}

pub trait Shape: Debug + Send + Sync {
    fn match_shape<'a>(&self, stream: TokenStream<'a>, context: &mut dyn MatchContext) -> MatchResult<'a>;

    fn adjacency(&self) -> AdjacencyConstraint {
        AdjacencyConstraint::None
    }

    fn complete<'a>(
        &self,
        _stream: TokenStream<'a>,
        _cursor: SourceLocation,
    ) -> CompletionFuture<'a> {
        ()
    }
}

// Matcher Trait
/// Defines how to match a single `TokenTree`.
/// Implementations exist for `AtomKind` (match by kind), `&str` (match by text), and `Delimiter` (match by delimiter type).
pub trait Matcher: Debug + Send + Sync {
    fn matches(&self, tree: &TokenTree) -> bool;
}

impl Matcher for AtomKind {
    fn matches(&self, tree: &TokenTree) -> bool {
        match tree {
            TokenTree::Token(token) => token.kind == *self,
            _ => false,
        }
    }
}

impl Matcher for &str {
    fn matches(&self, tree: &TokenTree) -> bool {
        match tree {
            TokenTree::Token(token) => token.text == *self,
            _ => false,
        }
    }
}

impl Matcher for Delimiter {
    fn matches(&self, tree: &TokenTree) -> bool {
        match tree {
            TokenTree::Delimited(d, _, _) => d.kind == self.kind,
            _ => false,
        }
    }
}

// Primitives

// term
/// Matches a single token or tree using the provided `Matcher`.
/// Implicitly skips leading whitespace.
#[derive(Debug)]
pub struct Term<M: Matcher>(pub M);

impl<M: Matcher> Shape for Term<M> {
    fn match_shape<'a>(&self, stream: TokenStream<'a>, _context: &mut dyn MatchContext) -> MatchResult<'a> {
        let mut current_stream = stream;

        // Skip whitespace
        while let Some(tree) = current_stream.first() {
            if let TokenTree::Token(token) = tree {
                if token.kind == AtomKind::Whitespace {
                    current_stream = current_stream.advance(1);
                    continue;
                }
            }
            break;
        }

        if let Some(tree) = current_stream.first() {
            if self.0.matches(tree) {
                return Ok((tree.clone(), current_stream.advance(1)));
            }
        }

        Err(())
    }
}

pub fn term<M: Matcher + 'static>(matcher: M) -> impl Shape {
    Term(matcher)
}

// seq
/// Matches shape `A` followed by shape `B`.
/// Implicitly skips whitespace between `A` and `B` (because `B`'s first term will skip it).
#[derive(Debug)]
pub struct Seq<A, B>(pub A, pub B);

impl<A: Shape, B: Shape> Shape for Seq<A, B> {
    fn match_shape<'a>(&self, stream: TokenStream<'a>, context: &mut dyn MatchContext) -> MatchResult<'a> {
        let (res_a, stream_after_a) = self.0.match_shape(stream, context)?;
        let (res_b, stream_after_b) = self.1.match_shape(stream_after_a, context)?;
        Ok((TokenTree::Group(vec![res_a, res_b]), stream_after_b))
    }
}

pub fn seq<A: Shape + 'static, B: Shape + 'static>(a: A, b: B) -> impl Shape {
    Seq(a, b)
}

// choice
/// Ordered choice: tries to match `A`, if it fails, tries to match `B`.
#[derive(Debug)]
pub struct Choice<A, B>(pub A, pub B);

impl<A: Shape, B: Shape> Shape for Choice<A, B> {
    fn match_shape<'a>(&self, stream: TokenStream<'a>, context: &mut dyn MatchContext) -> MatchResult<'a> {
        match self.0.match_shape(stream.clone(), context) {
            Ok(res) => Ok(res),
            Err(_) => self.1.match_shape(stream, context),
        }
    }
}

pub fn choice<A: Shape + 'static, B: Shape + 'static>(a: A, b: B) -> impl Shape {
    Choice(a, b)
}

// rep
/// Matches shape `A` zero or more times.
#[derive(Debug)]
pub struct Rep<A>(pub A);

impl<A: Shape> Shape for Rep<A> {
    fn match_shape<'a>(&self, stream: TokenStream<'a>, context: &mut dyn MatchContext) -> MatchResult<'a> {
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
}

pub fn rep<A: Shape + 'static>(a: A) -> impl Shape {
    Rep(a)
}

// enter
/// Matches a delimited group (e.g., `(...)`) and then matches the `inner` shape against the content of that group.
/// Implicitly skips leading whitespace before the group.
#[derive(Debug)]
pub struct Enter<S>(pub Delimiter, pub S);

impl<S: Shape> Shape for Enter<S> {
    fn match_shape<'a>(&self, stream: TokenStream<'a>, context: &mut dyn MatchContext) -> MatchResult<'a> {
        // 1. Match delimiter (skipping whitespace)
        let mut current_stream = stream;

        while let Some(tree) = current_stream.first() {
            if let TokenTree::Token(token) = tree {
                if token.kind == AtomKind::Whitespace {
                    current_stream = current_stream.advance(1);
                    continue;
                }
            }
            break;
        }

        if let Some(TokenTree::Delimited(d, content, _)) = current_stream.first() {
            if d.kind == self.0.kind {
                // 2. Create new stream from content
                let inner_stream = TokenStream::new(content);

                // 3. Match inner
                let (res, remaining_inner) = self.1.match_shape(inner_stream, context)?;

                // 4. Ensure inner consumed everything (Implicit Exit/End)
                let mut check_stream = remaining_inner;
                while let Some(tree) = check_stream.first() {
                    if let TokenTree::Token(token) = tree {
                        if token.kind == AtomKind::Whitespace {
                            check_stream = check_stream.advance(1);
                            continue;
                        }
                    }
                    // Found non-whitespace, so inner didn't consume everything
                    return Err(());
                }

                return Ok((res, current_stream.advance(1)));
            }
        }

        Err(())
    }
}

pub fn enter<S: Shape + 'static>(delimiter: Delimiter, inner: S) -> impl Shape {
    Enter(delimiter, inner)
}

// adjacent
/// Matches shape `A` followed by shape `B` with **no** intervening whitespace.
/// Used for tight binding (e.g., `obj.prop`).
#[derive(Debug)]
pub struct Adjacent<A, B>(pub A, pub B);

impl<A: Shape, B: Shape> Shape for Adjacent<A, B> {
    fn match_shape<'a>(&self, stream: TokenStream<'a>, context: &mut dyn MatchContext) -> MatchResult<'a> {
        let (res_a, stream_after_a) = self.0.match_shape(stream, context)?;

        // Check for whitespace at the start of stream_after_a
        if let Some(TokenTree::Token(token)) = stream_after_a.first() {
            if token.kind == AtomKind::Whitespace {
                return Err(());
            }
        }

        let (res_b, stream_after_b) = self.1.match_shape(stream_after_a, context)?;
        Ok((TokenTree::Group(vec![res_a, res_b]), stream_after_b))
    }
}

pub fn adjacent<A: Shape + 'static, B: Shape + 'static>(a: A, b: B) -> impl Shape {
    Adjacent(a, b)
}

// empty
#[derive(Debug)]
pub struct Empty;

impl Shape for Empty {
    fn match_shape<'a>(&self, stream: TokenStream<'a>, _context: &mut dyn MatchContext) -> MatchResult<'a> {
        Ok((TokenTree::Empty, stream))
    }
}

pub fn empty() -> impl Shape {
    Empty
}

// end
#[derive(Debug)]
pub struct End;

impl Shape for End {
    fn match_shape<'a>(&self, stream: TokenStream<'a>, _context: &mut dyn MatchContext) -> MatchResult<'a> {
        let mut current_stream = stream;
        while let Some(tree) = current_stream.first() {
            if let TokenTree::Token(token) = tree {
                if token.kind == AtomKind::Whitespace {
                    current_stream = current_stream.advance(1);
                    continue;
                }
            }
            return Err(());
        }
        Ok((TokenTree::Empty, current_stream))
    }
}

pub fn end() -> impl Shape {
    End
}

// expr
#[derive(Debug)]
pub struct Expr(pub Precedence);

impl Shape for Expr {
    fn match_shape<'a>(&self, stream: TokenStream<'a>, context: &mut dyn MatchContext) -> MatchResult<'a> {
        context.parse_expression(stream, self.0)
    }
}

pub fn expr(precedence: Precedence) -> impl Shape {
    Expr(precedence)
}

// Derived

pub fn opt<A: Shape + 'static>(a: A) -> impl Shape {
    choice(a, empty())
}

pub fn separated<A: Shape + 'static + Clone, S: Shape + 'static + Clone>(
    item: A,
    sep: S,
) -> impl Shape {
    // seq(item, rep(seq(sep, item)))
    seq(item.clone(), rep(seq(sep, item)))
}

pub fn joined<A: Shape + 'static + Clone>(a: A) -> impl Shape {
    // seq(a, rep(adjacent(empty(), a)))
    seq(a.clone(), rep(adjacent(empty(), a)))
}
