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
        _stream: TokenStream<'a>,
        _precedence: Precedence,
    ) -> MatchResult<'a> {
        Err(())
    }
}

pub trait Shape: Debug + Send + Sync {
    fn match_shape<'a>(
        &self,
        stream: TokenStream<'a>,
        context: &mut dyn MatchContext,
    ) -> MatchResult<'a>;

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
        if let Some(TokenTree::Token(token)) = stream_after_a.first() {
            if token.kind == AtomKind::Whitespace {
                return Err(());
            }
        }

        let (res_b, stream_after_b) = self.1.match_shape(stream_after_a, context)?;
        Ok((TokenTree::Group(vec![res_a, res_b]), stream_after_b))
    }
}

pub fn adjacent<A: Shape, B: Shape>(a: A, b: B) -> Adjacent<A, B> {
    Adjacent(a, b)
}

// empty
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

pub fn end() -> End {
    End
}

// expr
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

pub fn opt<A: Shape + Clone>(a: A) -> Choice<A, Empty> {
    choice(a, empty())
}

pub fn separated<A: Shape + Clone, S: Shape + Clone>(item: A, sep: S) -> Seq<A, Rep<Seq<S, A>>> {
    // seq(item, rep(seq(sep, item)))
    seq(item.clone(), rep(seq(sep, item)))
}

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
