use crate::atom::AtomKind;
use crate::language::Language;
use crate::r#macro::{ExpansionResult, MacroContext};
use crate::shape::{Associativity, MatchContext, MatchResult, Precedence};
use crate::token::{TokenStream, TokenTree};

pub struct Parser<'a, L: Language> {
    #[allow(dead_code)] // stream in struct might be used for initial entry point
    stream: TokenStream<'a>,
    language: &'a L,
}

impl<'a, L: Language> Parser<'a, L> {
    pub fn new(stream: TokenStream<'a>, language: &'a L) -> Self {
        Self { stream, language }
    }

    pub fn parse(&mut self) -> Result<TokenTree, String> {
        let (tree, _) = self
            .parse_expression(self.stream.clone(), Precedence(0))
            .map_err(|_| "Parse failed".to_string())?;
        Ok(tree)
    }

    fn parse_head<'s>(&mut self, stream: TokenStream<'s>) -> MatchResult<'s> {
        let mut current_stream = stream;

        // Skip whitespace
        while let Some(tree) = current_stream.first() {
            if let TokenTree::Token(token) = tree {
                if token.kind == AtomKind::Whitespace {
                    current_stream = current_stream.advance(1);
                    continue;
                }
            }
            break;
        }

        // Check for prefix macros
        let next_token_text = if let Some(TokenTree::Token(token)) = current_stream.first() {
            Some(token.text.as_str())
        } else {
            None
        };

        if let Some(text) = next_token_text {
            for mac in self.language.macros() {
                if !mac.is_operator() && mac.name() == text {
                    // Found prefix macro
                    let stream_after_name = current_stream.advance(1); // Consume name

                    // Match arguments
                    let (args, next_stream) =
                        mac.signature().match_shape(stream_after_name, self)?;

                    let context = MacroContext;
                    match mac.expand(args, None, &context) {
                        ExpansionResult::Ok(expanded) => return Ok((expanded, next_stream)),
                        ExpansionResult::Error(_) => return Err(()),
                    }
                }
            }
        }

        // If no macro, consume one token/tree as a term
        if let Some(tree) = current_stream.first() {
            Ok((tree.clone(), current_stream.advance(1)))
        } else {
            Err(())
        }
    }
}

impl<'a, L: Language> MatchContext for Parser<'a, L> {
    fn parse_expression<'s>(
        &mut self,
        stream: TokenStream<'s>,
        min_prec: Precedence,
    ) -> MatchResult<'s> {
        let (mut lhs, mut current_stream) = self.parse_head(stream)?;

        loop {
            let mut matched_op = None;

            // Peek at next token (skipping whitespace)
            let mut peek_stream = current_stream.clone();
            while let Some(tree) = peek_stream.first() {
                if let TokenTree::Token(token) = tree {
                    if token.kind == AtomKind::Whitespace {
                        peek_stream = peek_stream.advance(1);
                        continue;
                    }
                }
                break;
            }

            let next_token_text = if let Some(TokenTree::Token(token)) = peek_stream.first() {
                Some(token.text.as_str())
            } else {
                None
            };

            if let Some(text) = next_token_text {
                for mac in self.language.macros() {
                    if mac.is_operator() && mac.name() == text {
                        if mac.precedence() < min_prec {
                            continue;
                        }
                        if mac.precedence() == min_prec
                            && mac.associativity() == Associativity::Left
                        {
                            continue;
                        }

                        matched_op = Some(mac);
                        break;
                    }
                }
            }

            if let Some(mac) = matched_op {
                // Consume operator
                let stream_after_op = peek_stream.advance(1);

                // Match arguments
                // We pass `self` as context!
                let (args, next_stream) = mac.signature().match_shape(stream_after_op, self)?;

                current_stream = next_stream;

                // Expand
                let context = MacroContext;
                match mac.expand(args, Some(lhs.clone()), &context) {
                    ExpansionResult::Ok(expanded) => {
                        lhs = expanded;
                    }
                    ExpansionResult::Error(_) => return Err(()),
                }
            } else {
                break;
            }
        }

        Ok((lhs, current_stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::r#macro::{ExpansionResult, Macro, MacroContext};
    use crate::mock::MockLanguage;
    use crate::shape::{Precedence, Shape, expr, recover, seq, term};
    use crate::token::TokenTree;

    #[derive(Debug)]
    struct PlusMacro {
        shape: Box<dyn Shape>,
    }

    impl PlusMacro {
        fn new() -> Self {
            Self {
                shape: Box::new(expr(Precedence(10))),
            }
        }
    }

    impl Macro for PlusMacro {
        fn name(&self) -> &str {
            "+"
        }

        fn signature(&self) -> &dyn Shape {
            self.shape.as_ref()
        }

        fn expand(
            &self,
            args: TokenTree,
            lhs: Option<TokenTree>,
            _context: &MacroContext,
        ) -> ExpansionResult {
            let lhs = lhs.unwrap();
            let rhs = args;
            ExpansionResult::Ok(TokenTree::Group(vec![lhs, rhs]))
        }

        fn is_operator(&self) -> bool {
            true
        }

        fn precedence(&self) -> Precedence {
            Precedence(10)
        }
    }

    #[test]
    fn test_parse_simple_infix() {
        let lang = MockLanguage::new()
            .with_symbol("+")
            .with_macro(Box::new(PlusMacro::new()));

        let input = "a + b";
        let trees = lex(input, &lang);
        let stream = TokenStream::new(&trees);
        let mut parser = Parser::new(stream, &lang);

        let result = parser.parse().unwrap();

        // Expected: Group(a, b)
        if let TokenTree::Group(items) = result {
            assert_eq!(items.len(), 2);
            if let TokenTree::Token(t) = &items[0] {
                assert_eq!(t.text, "a");
            } else {
                panic!("Expected token a");
            }
            if let TokenTree::Token(t) = &items[1] {
                assert_eq!(t.text, "b");
            } else {
                panic!("Expected token b");
            }
        } else {
            panic!("Expected Group, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_precedence() {
        // a + b + c -> (a + b) + c (Left associative default)
        let lang = MockLanguage::new()
            .with_symbol("+")
            .with_macro(Box::new(PlusMacro::new()));

        let input = "a + b + c";
        let trees = lex(input, &lang);
        let stream = TokenStream::new(&trees);
        let mut parser = Parser::new(stream, &lang);

        let result = parser.parse().unwrap();

        // Expected: Group(Group(a, b), c)
        if let TokenTree::Group(items) = result {
            assert_eq!(items.len(), 2);
            // items[0] should be Group(a, b)
            if let TokenTree::Group(inner) = &items[0] {
                assert_eq!(inner.len(), 2);
                if let TokenTree::Token(t) = &inner[0] {
                    assert_eq!(t.text, "a");
                }
                if let TokenTree::Token(t) = &inner[1] {
                    assert_eq!(t.text, "b");
                }
            } else {
                panic!("Expected inner group");
            }

            // items[1] should be c
            if let TokenTree::Token(t) = &items[1] {
                assert_eq!(t.text, "c");
            } else {
                panic!("Expected token c");
            }
        } else {
            panic!("Expected Group, got {:?}", result);
        }
    }

    #[test]
    fn test_recover() {
        // A macro that expects "foo" then "bar".
        // If it fails, it recovers until ";".

        #[derive(Debug)]
        struct RecoverMacro {
            shape: Box<dyn Shape>,
        }

        impl RecoverMacro {
            fn new() -> Self {
                Self {
                    // Expect "foo" "bar", recover until ";"
                    shape: Box::new(recover(seq(term("foo"), term("bar")), ";")),
                }
            }
        }

        impl Macro for RecoverMacro {
            fn name(&self) -> &str {
                "stmt"
            }
            fn signature(&self) -> &dyn Shape {
                self.shape.as_ref()
            }
            fn expand(
                &self,
                args: TokenTree,
                _lhs: Option<TokenTree>,
                _context: &MacroContext,
            ) -> ExpansionResult {
                // args will be the result of recover.
                // If it failed, it will be TokenTree::Error.
                ExpansionResult::Ok(args)
            }
        }

        let lang = MockLanguage::new()
            .with_symbol(";")
            .with_macro(Box::new(RecoverMacro::new()));

        // Input: stmt foo baz ;
        // "foo" matches. "bar" fails (found "baz").
        // recover should skip "baz" and stop at ";".
        // Result should be Error.

        let input = "stmt foo baz ;";
        let trees = lex(input, &lang);
        let stream = TokenStream::new(&trees);
        let mut parser = Parser::new(stream, &lang);

        let result = parser.parse().unwrap();

        if let TokenTree::Error(msg) = result {
            assert!(msg.contains("skipped"));
        } else {
            panic!("Expected Error, got {:?}", result);
        }
    }
}
