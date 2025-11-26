#![allow(clippy::collapsible_if)]
use crate::atom::AtomKind;
use crate::language::Language;
use crate::scoping::ScopeStack;
use crate::shape::{CompletionItem, CompletionKind};
use crate::token::{Token, TokenTree};

pub fn find_completions(
    tokens: &[TokenTree],
    language: &(impl Language + ?Sized),
    offset: usize,
) -> Vec<CompletionItem> {
    let mut scope = ScopeStack::new();

    // Populate scope stack up to the cursor
    language
        .binding_pass()
        .collect_scope_at(tokens, offset, &mut scope);

    // Determine prefix to calculate delete_backwards
    let mut delete_backwards = 0;
    if let Some(token) = find_token_at_offset(tokens, offset) {
        if matches!(token.kind, AtomKind::Identifier) {
            // If cursor is at the end or inside the identifier
            if token.location.span.offset() + token.location.span.len() >= offset {
                // Calculate how much of the identifier is before the cursor
                let len = offset.saturating_sub(token.location.span.offset());
                delete_backwards = len;
            }
        }
    }

    let mut items = Vec::new();

    // Add variables from scope
    for name in scope.names() {
        items.push(CompletionItem {
            label: name,
            kind: CompletionKind::Variable,
            detail: None,
            delete_backwards,
        });
    }

    items
}

fn find_token_at_offset(tokens: &[TokenTree], offset: usize) -> Option<&Token> {
    for tree in tokens {
        match tree {
            TokenTree::Token(t) => {
                let span = t.location.span;
                // Check if offset is within or at the end of the token
                if span.offset() <= offset && offset <= span.offset() + span.len() {
                    return Some(t);
                }
            }
            TokenTree::Delimited(_, children, loc, _) => {
                if loc.span.offset() <= offset && offset <= loc.span.offset() + loc.span.len() {
                    if let Some(t) = find_token_at_offset(children, offset) {
                        return Some(t);
                    }
                }
            }
            TokenTree::Group(children) => {
                if let Some(t) = find_token_at_offset(children, offset) {
                    return Some(t);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::mock::MockLanguage;

    #[test]
    fn test_completion_simple() {
        let lang = MockLanguage::new().with_keyword_binding("let");
        let input = "let x = 1; ";
        // Cursor at end
        let offset = input.len();

        let tokens = lex(input, &lang);
        let completions = find_completions(&tokens, &lang, offset);

        assert!(completions.iter().any(|c| c.label == "x"));
    }

    #[test]
    fn test_completion_inside_block() {
        let lang = MockLanguage::new().with_keyword_binding("let");
        // Note: MockLanguage uses "paren" delimiter by default, not "brace".
        // I need to update MockLanguage or use parens.
        // Let's use parens for now as MockLanguage has them.
        let input = "let x = 1; ( let y = 2; ";
        // Cursor at end, inside unclosed paren
        let offset = input.len();

        let tokens = lex(input, &lang);
        let completions = find_completions(&tokens, &lang, offset);

        assert!(completions.iter().any(|c| c.label == "x"));
        assert!(completions.iter().any(|c| c.label == "y"));
    }

    #[test]
    fn test_completion_shadowing() {
        let lang = MockLanguage::new().with_keyword_binding("let");
        let input = "let x = 1; ( let x = 2; ";
        let offset = input.len();

        let tokens = lex(input, &lang);
        let completions = find_completions(&tokens, &lang, offset);

        let x_count = completions.iter().filter(|c| c.label == "x").count();
        assert!(x_count >= 1);
    }
}
