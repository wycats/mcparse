use crate::atom::Atom;
use crate::r#macro::Macro;
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
    // variable_rules to be added later
}
