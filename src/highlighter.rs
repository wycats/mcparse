use crate::token::Token;
use std::fmt::Debug;
use owo_colors::OwoColorize;

#[derive(Debug, Clone, Copy)]
pub enum HighlightStyle {
    Keyword,
    String,
    Number,
    Comment,
    Operator,
    Punctuation,
    Variable,
    Function,
    None,
}

pub trait Highlighter: Debug + Send + Sync {
    fn highlight(&mut self, token: &Token, style: HighlightStyle);
}

#[derive(Debug)]
pub struct ANSIHighlighter;

impl Highlighter for ANSIHighlighter {
    fn highlight(&mut self, token: &Token, style: HighlightStyle) {
        let text = &token.text;
        match style {
            HighlightStyle::Keyword => print!("{}", text.blue()),
            HighlightStyle::String => print!("{}", text.green()),
            HighlightStyle::Number => print!("{}", text.yellow()),
            HighlightStyle::Comment => print!("{}", text.bright_black()),
            HighlightStyle::Operator => print!("{}", text.red()),
            HighlightStyle::Punctuation => print!("{}", text.white()),
            HighlightStyle::Variable => print!("{}", text.cyan()),
            HighlightStyle::Function => print!("{}", text.magenta()),
            HighlightStyle::None => print!("{}", text),
        }
    }
}
