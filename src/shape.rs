use crate::token::{TokenStream, TokenTree, SourceLocation};
use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub enum AdjacencyConstraint {
    None,
    Required,
    Forbidden,
}

pub type MatchResult<'a> = Result<(TokenTree, TokenStream<'a>), ()>;

// Placeholder for CompletionFuture
pub type CompletionFuture<'a> = (); 

pub trait Shape: Debug + Send + Sync {
    fn match_shape<'a>(&self, stream: TokenStream<'a>) -> MatchResult<'a>;

    fn adjacency(&self) -> AdjacencyConstraint {
        AdjacencyConstraint::None
    }

    fn complete<'a>(&self, _stream: TokenStream<'a>, _cursor: SourceLocation) -> CompletionFuture<'a> {
        ()
    }
}
