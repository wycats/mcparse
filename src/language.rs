pub use crate::atom::{Atom, AtomKind};
use crate::r#macro::Macro;
use crate::scoping::{BindingPass, ReferencePass};
use crate::shape::CompletionItem;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delimiter {
    pub kind: &'static str,
    pub open: &'static str,
    pub close: &'static str,
}

pub trait Language: Debug + Send + Sync {
    fn atoms(&self) -> &[Box<dyn Atom>];
    fn delimiters(&self) -> &[Delimiter];
    fn macros(&self) -> &[Box<dyn Macro>];
    fn binding_pass(&self) -> &dyn BindingPass;
    fn reference_pass(&self) -> &dyn ReferencePass;

    fn complete(&self, input: &str, offset: usize) -> Vec<CompletionItem> {
        let tokens = crate::lexer::lex(input, self);
        crate::completion::find_completions(&tokens, self, offset)
    }
}
