use crate::atom::{Atom, AtomKind, VariableRole};
use crate::r#macro::Macro;
use crate::token::Token;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delimiter {
    pub kind: &'static str,
    pub open: &'static str,
    pub close: &'static str,
}

pub trait VariableRules: Debug + Send + Sync {
    /// Classifies an identifier token as a Binding, Reference, or None based on the previous token.
    /// This allows for context-sensitive lexing (e.g., "let x" vs "x = 1").
    fn classify(&self, previous_token: Option<&Token>, current_token: &Token) -> VariableRole;
}

#[derive(Debug, Default)]
pub struct PatternVariableRules {
    bind_after_keywords: Vec<String>,
}

impl PatternVariableRules {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_after_keyword(mut self, keyword: &str) -> Self {
        self.bind_after_keywords.push(keyword.to_string());
        self
    }
}

impl VariableRules for PatternVariableRules {
    fn classify(&self, previous_token: Option<&Token>, current_token: &Token) -> VariableRole {
        if !matches!(current_token.kind, AtomKind::Identifier(_)) {
            return VariableRole::None;
        }

        if let Some(prev) = previous_token {
            if let AtomKind::Keyword(ref k) = prev.kind {
                if self.bind_after_keywords.contains(k) {
                    return VariableRole::Binding;
                }
            }
        }

        VariableRole::Reference
    }
}

pub trait Language: Debug + Send + Sync {
    fn atoms(&self) -> &[Box<dyn Atom>];
    fn delimiters(&self) -> &[Delimiter];
    fn macros(&self) -> &[Box<dyn Macro>];
    fn variable_rules(&self) -> &dyn VariableRules;
}
