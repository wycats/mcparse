#[cfg(test)]
mod tests {
    use crate::atom::AtomKind;
    use crate::language::Delimiter;
    use crate::token::{SourceLocation, Token, TokenTree};
    use miette::SourceSpan;

    fn mock_token(text: &str) -> TokenTree {
        TokenTree::Token(Token {
            kind: AtomKind::Identifier(crate::atom::VariableRole::None),
            text: text.to_string(),
            location: SourceLocation {
                span: SourceSpan::new(0usize.into(), 0usize.into()),
            },
            atom_index: None,
        })
    }

    #[test]
    fn test_sexp_token() {
        let t = mock_token("hello");
        assert_eq!(t.to_sexp(), "\"hello\"");
    }

    #[test]
    fn test_sexp_group() {
        let t1 = mock_token("a");
        let t2 = mock_token("b");
        let group = TokenTree::Group(vec![t1, t2]);
        assert_eq!(group.to_sexp(), "(group \"a\" \"b\")");
    }

    #[test]
    fn test_sexp_delimited() {
        let t1 = mock_token("key");
        let d = Delimiter {
            kind: "brace",
            open: "{",
            close: "}",
        };
        let tree = TokenTree::Delimited(
            d,
            vec![t1],
            SourceLocation {
                span: SourceSpan::new(0usize.into(), 0usize.into()),
            },
        );
        assert_eq!(tree.to_sexp(), "(brace \"key\")");
    }
}
