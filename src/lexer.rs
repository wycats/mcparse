use crate::atom::AtomKind;
use crate::language::{Delimiter, Language};
use crate::token::{Cursor, SourceLocation, Token, TokenTree};
use miette::SourceSpan;

/// The entry point for the atomic lexer.
/// Converts a raw string into a list of `TokenTree`s, handling delimiters recursively.
pub fn lex(input: &str, language: &impl Language) -> Vec<TokenTree> {
    let cursor = Cursor::new(input);
    let (trees, _) = lex_group(cursor, language, None);
    trees
}

/// Recursively lexes a group of tokens until the input is exhausted or a closing delimiter is found.
fn lex_group<'a>(
    mut cursor: Cursor<'a>,
    language: &impl Language,
    terminator: Option<&Delimiter>,
) -> (Vec<TokenTree>, Cursor<'a>) {
    let mut trees = Vec::new();
    let mut previous_token: Option<Token> = None;

    'outer: while !cursor.rest.is_empty() {
        // 1. Check for terminator (close delimiter)
        if let Some(term) = terminator {
            if cursor.rest.starts_with(term.close) {
                return (trees, cursor);
            }
        }

        // 2. Check for openers (delimiters)
        for delim in language.delimiters() {
            if cursor.rest.starts_with(delim.open) {
                let start_offset = cursor.offset;
                let inner_cursor = cursor.advance(delim.open.len());
                let (inner_trees, next_cursor) = lex_group(inner_cursor, language, Some(delim));
                
                // Check if we found the closer
                if next_cursor.rest.starts_with(delim.close) {
                    let end_cursor = next_cursor.advance(delim.close.len());
                    let span = SourceSpan::new(start_offset.into(), (end_cursor.offset - start_offset).into());
                    let location = SourceLocation { span };
                    
                    trees.push(TokenTree::Delimited(delim.clone(), inner_trees, location));
                    cursor = end_cursor;
                    // Reset previous_token as we just finished a group
                    previous_token = None; 
                    continue 'outer;
                } else {
                    // Unclosed delimiter - treat as error or just stop?
                    // For now, we return what we have. The caller will see we didn't consume everything if they check.
                    // But here we are inside a recursive call.
                    // If we don't find the closer, we probably hit EOF.
                    return (trees, next_cursor);
                }
            }
        }

        // 3. Check for atoms
        let mut matched_atom = false;
        for atom in language.atoms() {
            if let Some((mut token, next_cursor)) = atom.parse(cursor) {
                // Apply variable rules
                if let AtomKind::Identifier(_) = token.kind {
                    let role = language.variable_rules().classify(previous_token.as_ref(), &token);
                    token.kind = AtomKind::Identifier(role);
                }

                trees.push(TokenTree::Token(token.clone()));
                
                // Update previous_token only if it's not whitespace
                if !matches!(token.kind, AtomKind::Whitespace) {
                    previous_token = Some(token);
                }
                
                cursor = next_cursor;
                matched_atom = true;
                break;
            }
        }

        if !matched_atom {
            // Skip one character to avoid infinite loop
             let char_len = cursor.rest.chars().next().unwrap().len_utf8();
             cursor = cursor.advance(char_len);
        }
    }

    (trees, cursor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockLanguage;
    use crate::atom::{AtomKind, VariableRole};

    #[test]
    fn test_lex_simple() {
        let lang = MockLanguage::new();
        let input = "foo bar";
        let trees = lex(input, &lang);
        
        assert_eq!(trees.len(), 3); // foo, space, bar
        
        if let TokenTree::Token(t) = &trees[0] {
            assert_eq!(t.text, "foo");
            assert!(matches!(t.kind, AtomKind::Identifier(VariableRole::Reference)));
        } else {
            panic!("Expected token");
        }
    }

    #[test]
    fn test_lex_binding() {
        let lang = MockLanguage::new().with_keyword_binding("let");
        let input = "let x";
        let trees = lex(input, &lang);
        
        // let, space, x
        assert_eq!(trees.len(), 3);
        
        if let TokenTree::Token(t) = &trees[0] {
            assert_eq!(t.text, "let");
            assert!(matches!(t.kind, AtomKind::Keyword(_)));
        }
        
        if let TokenTree::Token(t) = &trees[2] {
            assert_eq!(t.text, "x");
            assert!(matches!(t.kind, AtomKind::Identifier(VariableRole::Binding)));
        } else {
            panic!("Expected token x");
        }
    }

    #[test]
    fn test_lex_group() {
        let lang = MockLanguage::new();
        let input = "(foo)";
        let trees = lex(input, &lang);
        
        assert_eq!(trees.len(), 1);
        
        if let TokenTree::Delimited(delim, inner, _) = &trees[0] {
            assert_eq!(delim.kind, "paren");
            assert_eq!(inner.len(), 1); // foo
            if let TokenTree::Token(t) = &inner[0] {
                assert_eq!(t.text, "foo");
            }
        } else {
            panic!("Expected delimited");
        }
    }
}
