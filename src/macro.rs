use crate::shape::Shape;
use crate::token::TokenTree;
use std::fmt::Debug;

pub struct MacroContext; // Placeholder

pub enum ExpansionResult {
    Ok(TokenTree),
    Error(String), // Placeholder
}

pub trait Macro: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn signature(&self) -> &dyn Shape;
    fn expand(&self, input: TokenTree, context: &MacroContext) -> ExpansionResult;
    
    fn is_operator(&self) -> bool { false }
    // precedence to be added
}
