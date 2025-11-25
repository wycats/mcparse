#[cfg(test)]
mod tests {
    use crate::atom::AtomKind;
    use crate::shape::{
        CompletionKind, MatchContext, Matcher, NoOpMatchContext, Shape, choice, rep, seq, term,
    };
    use crate::token::{SourceLocation, Token, TokenStream, TokenTree};
    use miette::SourceSpan;

    fn mock_token(text: &str, offset: usize) -> TokenTree {
        TokenTree::Token(Token {
            kind: AtomKind::Identifier(crate::atom::VariableRole::None),
            text: text.to_string(),
            location: SourceLocation {
                span: SourceSpan::new(offset.into(), text.len().into()),
            },
        })
    }

    #[test]
    fn test_term_completion() {
        let t = mock_token("fun", 0);
        let trees = vec![t];
        let stream = TokenStream::new(&trees);
        let shape = term("function");
        let mut ctx = NoOpMatchContext;

        let items = shape.complete(stream, &mut ctx, 1); // Cursor at 'u'
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "function");
    }

    #[test]
    fn test_seq_completion_first() {
        let t = mock_token("fun", 0);
        let trees = vec![t];
        let stream = TokenStream::new(&trees);
        let shape = seq(term("function"), term("name"));
        let mut ctx = NoOpMatchContext;

        let items = shape.complete(stream, &mut ctx, 1);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "function");
    }

    #[test]
    fn test_seq_completion_second() {
        let t1 = mock_token("function", 0);
        let t2 = mock_token("na", 9); // "function " is 9 chars
        let trees = vec![t1, t2];
        let stream = TokenStream::new(&trees);
        let shape = seq(term("function"), term("name"));
        let mut ctx = NoOpMatchContext;

        let items = shape.complete(stream, &mut ctx, 10); // Cursor at 'a'
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "name");
    }

    #[test]
    fn test_choice_completion() {
        let t = mock_token("le", 0);
        let trees = vec![t];
        let stream = TokenStream::new(&trees);
        let shape = choice(term("let"), term("left"));
        let mut ctx = NoOpMatchContext;

        let items = shape.complete(stream, &mut ctx, 1);
        assert_eq!(items.len(), 2);
        // Order depends on implementation, but both should be there
        let labels: Vec<String> = items.iter().map(|i| i.label.clone()).collect();
        assert!(labels.contains(&"let".to_string()));
        assert!(labels.contains(&"left".to_string()));
    }
}
