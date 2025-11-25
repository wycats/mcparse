use crate::shape::{Associativity, Precedence, Shape};
use crate::token::TokenTree;
use std::fmt::Debug;

pub struct MacroContext; // Placeholder

#[derive(Debug, Clone)]
pub enum ExpansionResult {
    Ok(TokenTree),
    Error(String), // Placeholder
}

pub trait Macro: Debug + Send + Sync {
    fn name(&self) -> &str;
    
    /// The shape to match arguments from the stream.
    /// For a prefix macro, this matches arguments after the macro name.
    /// For an infix macro, this matches arguments after the operator.
    fn signature(&self) -> &dyn Shape;

    /// Expand the macro.
    /// `args`: The result of matching the signature.
    /// `lhs`: The left-hand side expression, if this is an operator being applied to it.
    fn expand(&self, args: TokenTree, lhs: Option<TokenTree>, context: &MacroContext) -> ExpansionResult;
    
    fn is_operator(&self) -> bool { false }
    fn precedence(&self) -> Precedence { Precedence(0) }
    fn associativity(&self) -> Associativity { Associativity::Left }
}
