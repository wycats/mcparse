use crate::atom::AtomKind;
use crate::token::{BindingId, TokenTree};
use std::collections::HashMap;
use std::fmt::Debug;

/// A stack of scopes for resolving variables.
#[derive(Debug, Default)]
pub struct ScopeStack {
    scopes: Vec<HashMap<String, BindingId>>,
    next_id: usize,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Global scope
            next_id: 0,
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: String) -> BindingId {
        let id = BindingId(self.next_id);
        self.next_id += 1;
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, id);
        }
        id
    }

    pub fn define_existing(&mut self, name: String, id: BindingId) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, id);
        }
    }

    pub fn resolve(&self, name: &str) -> Option<BindingId> {
        for scope in self.scopes.iter().rev() {
            if let Some(id) = scope.get(name) {
                return Some(*id);
            }
        }
        None
    }

    pub fn names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for scope in &self.scopes {
            for name in scope.keys() {
                names.push(name.clone());
            }
        }
        names
    }
}

/// A trait for identifying variable bindings in a token stream.
pub trait BindingPass: Debug + Send + Sync {
    /// Scans the token stream and identifies variable bindings.
    /// This method should look for patterns that define new variables (e.g., `let x`)
    /// and mark the corresponding tokens as bindings in the `ScopeStack`.
    fn identify_bindings(&self, tokens: &mut [TokenTree], scope: &mut ScopeStack);

    /// Scans the token stream to build the scope stack at a specific offset.
    /// Returns `true` if the offset was found within the tokens.
    fn collect_scope_at(&self, tokens: &[TokenTree], offset: usize, scope: &mut ScopeStack) -> bool {
        for token in tokens {
            match token {
                TokenTree::Token(t) => {
                    let span = t.location.span;
                    if span.offset() <= offset && offset < span.offset() + span.len() {
                        return true;
                    }
                }
                TokenTree::Delimited(_, children, loc, is_closed) => {
                    let span = loc.span;
                    let end_check = if *is_closed {
                        offset < span.offset() + span.len()
                    } else {
                        offset <= span.offset() + span.len()
                    };

                    if span.offset() <= offset && end_check {
                        scope.push();
                        if self.collect_scope_at(children, offset, scope) {
                            return true;
                        }
                        // If we are in the delimiter but not in a child, we are in the scope.
                        return true;
                    }
                }
                TokenTree::Group(children) => {
                    if self.collect_scope_at(children, offset, scope) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

/// A trait for resolving variable references in a token stream.
pub trait ReferencePass: Debug + Send + Sync {
    /// Scans the token stream and resolves variable references.
    /// This method should look for identifiers that refer to existing variables
    /// and link them to their definitions in the `ScopeStack`.
    fn resolve_references(&self, tokens: &mut [TokenTree], scope: &mut ScopeStack);
}

#[derive(Debug, Default)]
pub struct NoOpBindingPass;

impl BindingPass for NoOpBindingPass {
    fn identify_bindings(&self, _tokens: &mut [TokenTree], _scope: &mut ScopeStack) {}
}

#[derive(Debug, Default)]
pub struct NoOpReferencePass;

impl ReferencePass for NoOpReferencePass {
    fn resolve_references(&self, _tokens: &mut [TokenTree], _scope: &mut ScopeStack) {}
}

#[derive(Debug)]
pub struct SimpleBindingPass {
    keyword: String,
}

impl SimpleBindingPass {
    pub fn new(keyword: &str) -> Self {
        Self {
            keyword: keyword.to_string(),
        }
    }
}

impl BindingPass for SimpleBindingPass {
    fn identify_bindings(&self, tokens: &mut [TokenTree], scope: &mut ScopeStack) {
        let mut i = 0;
        while i < tokens.len() {
            // Handle recursion
            let is_container = matches!(tokens[i], TokenTree::Delimited(..) | TokenTree::Group(..));

            if is_container {
                match &mut tokens[i] {
                    TokenTree::Delimited(_, children, _, _) => {
                        scope.push();
                        self.identify_bindings(children, scope);
                        scope.pop();
                    }
                    TokenTree::Group(children) => {
                        self.identify_bindings(children, scope);
                    }
                    _ => unreachable!(),
                }
                i += 1;
                continue;
            }

            // Check for binding pattern: keyword <ident>
            let is_keyword = if let TokenTree::Token(t) = &tokens[i] {
                t.text == self.keyword
            } else {
                false
            };

            if is_keyword {
                // Look ahead for identifier, skipping whitespace
                let mut j = i + 1;
                while j < tokens.len() {
                    let is_whitespace = if let TokenTree::Token(t) = &tokens[j] {
                        matches!(t.kind, AtomKind::Whitespace)
                    } else {
                        false
                    };

                    if is_whitespace {
                        j += 1;
                        continue;
                    }

                    let is_identifier = if let TokenTree::Token(t) = &tokens[j] {
                        matches!(t.kind, AtomKind::Identifier)
                    } else {
                        false
                    };

                    if is_identifier {
                        // Found it! Now we need to mutate it.
                        if let TokenTree::Token(t) = &mut tokens[j] {
                            let name = t.text.clone();
                            let id = scope.define(name);
                            t.binding = Some(id);
                        }
                        break;
                    }

                    break;
                }
            }
            i += 1;
        }
    }

    fn collect_scope_at(&self, tokens: &[TokenTree], offset: usize, scope: &mut ScopeStack) -> bool {
        let mut i = 0;
        while i < tokens.len() {
            // Check if we hit the offset in the current token
            let current_token = &tokens[i];
            let span = match current_token {
                TokenTree::Token(t) => Some(t.location.span),
                TokenTree::Delimited(_, _, loc, _) => Some(loc.span),
                _ => None,
            };

            if let Some(span) = span {
                let is_inside = if let TokenTree::Delimited(_, _, _, is_closed) = current_token {
                    if *is_closed {
                        span.offset() <= offset && offset < span.offset() + span.len()
                    } else {
                        span.offset() <= offset && offset <= span.offset() + span.len()
                    }
                } else {
                    span.offset() <= offset && offset < span.offset() + span.len()
                };

                if is_inside {
                    // We are inside this token.
                    // If it's a delimiter, recurse.
                    if let TokenTree::Delimited(_, children, _, _) = current_token {
                        scope.push();
                        if self.collect_scope_at(children, offset, scope) {
                            return true;
                        }
                        // Inside delimiter but not in children -> return true (keep scope)
                        return true;
                    }
                    // If it's a token, we are at it. Return true.
                    return true;
                }
                // If we passed the offset, we can stop?
                // Only if we are sure the tokens are ordered.
                if span.offset() > offset {
                    return false;
                }
            }
            
            // Handle Group recursion (transparent)
            if let TokenTree::Group(children) = current_token {
                 if self.collect_scope_at(children, offset, scope) {
                     return true;
                 }
                 i += 1;
                 continue;
            }

            // Process bindings (same logic as identify_bindings but read-only)
            // Check for binding pattern: keyword <ident>
            let is_keyword = if let TokenTree::Token(t) = &tokens[i] {
                t.text == self.keyword
            } else {
                false
            };

            if is_keyword {
                // Look ahead for identifier
                let mut j = i + 1;
                while j < tokens.len() {
                    let is_whitespace = if let TokenTree::Token(t) = &tokens[j] {
                        matches!(t.kind, AtomKind::Whitespace)
                    } else {
                        false
                    };

                    if is_whitespace {
                        j += 1;
                        continue;
                    }

                    let is_identifier = if let TokenTree::Token(t) = &tokens[j] {
                        matches!(t.kind, AtomKind::Identifier)
                    } else {
                        false
                    };

                    if is_identifier {
                        // Found it! Define it in scope.
                        if let TokenTree::Token(t) = &tokens[j] {
                            scope.define(t.text.clone());
                        }
                        break;
                    }
                    break;
                }
            }
            i += 1;
        }
        false
    }
}

#[derive(Debug, Default)]
pub struct SimpleReferencePass;

impl ReferencePass for SimpleReferencePass {
    fn resolve_references(&self, tokens: &mut [TokenTree], scope: &mut ScopeStack) {
        for tree in tokens {
            match tree {
                TokenTree::Delimited(_, children, _, _) => {
                    scope.push();
                    self.resolve_references(children, scope);
                    scope.pop();
                }
                TokenTree::Group(children) => {
                    self.resolve_references(children, scope);
                }
                TokenTree::Token(token) => {
                    if matches!(token.kind, AtomKind::Identifier) {
                        if let Some(id) = token.binding {
                            // This is a definition (marked by BindingPass)
                            scope.define_existing(token.text.clone(), id);
                        } else {
                            // This is a reference
                            if let Some(id) = scope.resolve(&token.text) {
                                token.binding = Some(id);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Helper function to run both passes on a token stream.
pub fn scope_tokens(tokens: &mut [TokenTree], language: &impl crate::Language) {
    let mut scope = ScopeStack::new();
    language
        .binding_pass()
        .identify_bindings(tokens, &mut scope);

    let mut scope = ScopeStack::new();
    language
        .reference_pass()
        .resolve_references(tokens, &mut scope);
}
