//! # McParse
//!
//! McParse is a parser toolkit designed for building modern, robust, and interactive language tools.
//! It differs from traditional parser generators by focusing on:
//!
//! *   **Resilience**: Parsing continues even in the presence of syntax errors.
//! *   **Interactivity**: Built-in support for syntax highlighting and code completion.
//! *   **Extensibility**: A powerful macro system that allows the language to evolve.
//!
//! ## The Pipeline
//!
//! 1.  **Atomic Lexing**: Raw text is converted into a tree of `TokenTree`s.
//! 2.  **Macro Expansion**: Macros are expanded to transform the tree.
//! 3.  **Shape Matching**: A grammar (defined by "Shapes") is matched against the tree.
//!
//! ## Getting Started
//!
//! The best way to learn McParse is by reading [The McParse Book](https://wycats.github.io/mcparse/book/).
//!
//! ## Example
//!
//! ```rust
//! use mcparse::{define_atom, atom::AtomKind, token::Token, highlighter::HighlightStyle};
//!
//! define_atom! {
//!     struct Number;
//!     kind = AtomKind::Number;
//!     parse(input) {
//!         let len = input.rest.chars().take_while(|c| c.is_ascii_digit()).map(|c| c.len_utf8()).sum();
//!         if len > 0 {
//!             Some((Token::new(AtomKind::Number, &input.rest[..len], input.offset), input.advance(len)))
//!         } else {
//!             None
//!         }
//!     }
//!     highlight(token, h) { h.highlight(token, HighlightStyle::Number); }
//! }
//! ```

pub mod atom;
pub mod atoms;
pub mod completion;
pub mod highlighter;
pub mod incremental;
pub mod language;
pub mod lexer;
pub mod r#macro;
mod macros;
#[cfg(test)]
pub mod mock;
pub mod parser;
pub mod scoping;
pub mod shape;
pub mod token;

pub use atom::{Atom, AtomKind};
pub use highlighter::{HighlightStyle, Highlighter};
pub use incremental::{GreenTree, RedNode, TextEdit, incremental_relex};
pub use language::Language;
pub use r#macro::{ExpansionResult, Macro, MacroContext};
pub use parser::Parser;
pub use shape::{
    AdjacencyConstraint, MatchContext, MatchResult, Shape, adjacent, choice, empty, end, enter,
    expr, joined, opt, recover, rep, separated, seq, term,
};
pub use token::{Cursor, SourceLocation, Token, TokenTree};

#[cfg(test)]
mod atoms_tests;
#[cfg(test)]
mod macro_tests;
#[cfg(test)]
mod shape_tests;
#[cfg(test)]
mod token_tests;

#[cfg(doctest)]
mod book_tests {
    #[doc = include_str!("../docs/book/src/quickstart.md")]
    pub struct QuickStart;

    #[doc = include_str!("../docs/book/src/concepts.md")]
    pub struct Concepts;

    #[doc = include_str!("../docs/book/src/tutorial/json/atoms.md")]
    pub struct JsonAtoms;

    #[doc = include_str!("../docs/book/src/tutorial/json/shapes.md")]
    pub struct JsonShapes;

    #[doc = include_str!("../docs/book/src/tutorial/scripting/index.md")]
    pub struct ScriptingIntro;

    #[doc = include_str!("../docs/book/src/tutorial/scripting/macros.md")]
    pub struct ScriptingMacros;

    #[doc = include_str!("../docs/book/src/tutorial/scripting/expressions.md")]
    pub struct ScriptingExpressions;

    #[doc = include_str!("../docs/book/src/advanced/error-recovery.md")]
    pub struct ErrorRecovery;

    #[doc = include_str!("../docs/book/src/advanced/custom-atoms.md")]
    pub struct CustomAtoms;

    #[doc = include_str!("../docs/book/src/advanced/custom-shapes.md")]
    pub struct CustomShapes;

    #[doc = include_str!("../docs/book/src/reference/atoms.md")]
    pub struct RefAtoms;

    #[doc = include_str!("../docs/book/src/reference/shapes.md")]
    pub struct RefShapes;

    #[doc = include_str!("../docs/book/src/reference/macros.md")]
    pub struct RefMacros;

    #[doc = include_str!("../docs/book/src/reference/hygiene.md")]
    pub struct RefHygiene;

    #[doc = include_str!("../docs/book/src/reference/highlighting.md")]
    pub struct RefHighlighting;

    #[doc = include_str!("../docs/book/src/cookbook/index.md")]
    pub struct Cookbook;

    #[doc = include_str!("../docs/book/src/advanced/contextual-keywords.md")]
    pub struct ContextualKeywords;
}
