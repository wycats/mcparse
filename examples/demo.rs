use mcparse::atom::Atom;
use mcparse::highlighter::ANSIHighlighter;
use mcparse::mock::{IdentifierAtom, WhitespaceAtom};
use mcparse::token::Cursor;
use miette::{Diagnostic, NamedSource, Result};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("Parse Error")]
#[diagnostic(code(mcparse::parse_error))]
struct ParseError {
    #[source_code]
    src: NamedSource<String>,
    #[label("Unexpected character")]
    span: miette::SourceSpan,
}

fn main() -> Result<()> {
    let input = "hello world";
    let src = NamedSource::new("demo.mc", input.to_string());

    let atoms: Vec<Box<dyn Atom>> = vec![
        Box::new(WhitespaceAtom),
        Box::new(IdentifierAtom),
    ];

    let mut cursor = Cursor::new(input);
    let mut highlighter = ANSIHighlighter;

    println!("Parsing: '{}'", input);

    while !cursor.rest.is_empty() {
        let mut matched = false;
        for atom in &atoms {
            if let Some((token, next_cursor)) = atom.parse(cursor) {
                atom.highlight(&token, &mut highlighter);
                cursor = next_cursor;
                matched = true;
                break;
            }
        }

        if !matched {
            // Report error
            return Err(ParseError {
                src,
                span: (cursor.offset, 1).into(),
            }.into());
        }
    }
    println!(); // Newline after highlighting

    // Trigger an error
    let input_err = "hello 123";
    println!("\nParsing with error: '{}'", input_err);
    let src_err = NamedSource::new("error.mc", input_err.to_string());
    let mut cursor_err = Cursor::new(input_err);
    
    while !cursor_err.rest.is_empty() {
        let mut matched = false;
        for atom in &atoms {
            if let Some((token, next_cursor)) = atom.parse(cursor_err) {
                atom.highlight(&token, &mut highlighter);
                cursor_err = next_cursor;
                matched = true;
                break;
            }
        }

        if !matched {
             println!();
             return Err(ParseError {
                src: src_err,
                span: (cursor_err.offset, 1).into(),
            }.into());
        }
    }

    Ok(())
}
