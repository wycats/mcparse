use crate::atom::{Atom, AtomKind};
use crate::atoms::{KeywordAtom, RegexAtom};
use crate::token::Cursor;

#[test]
fn test_regex_atom_whitespace() {
    let atom = RegexAtom::new(AtomKind::Whitespace, r"\s+");
    let input = "   hello";
    let cursor = Cursor::new(input);

    let (token, next) = atom.parse(cursor).expect("Should match whitespace");
    assert_eq!(token.kind, AtomKind::Whitespace);
    assert_eq!(token.text, "   ");
    assert_eq!(token.location.span.offset(), 0);
    assert_eq!(token.location.span.len(), 3);
    assert_eq!(next.offset, 3);
    assert_eq!(next.rest, "hello");
}

#[test]
fn test_regex_atom_identifier() {
    let atom = RegexAtom::new(
        AtomKind::Identifier(crate::atom::VariableRole::None),
        r"[a-zA-Z_]\w*",
    );
    let input = "variable_123 = 5";
    let cursor = Cursor::new(input);

    let (token, next) = atom.parse(cursor).expect("Should match identifier");
    assert!(matches!(token.kind, AtomKind::Identifier(_)));
    assert_eq!(token.text, "variable_123");
    assert_eq!(token.location.span.len(), 12);
    assert_eq!(next.rest, " = 5");
}

#[test]
fn test_regex_atom_no_match() {
    let atom = RegexAtom::new(AtomKind::Number, r"\d+");
    let input = "abc";
    let cursor = Cursor::new(input);

    assert!(atom.parse(cursor).is_none());
}

#[test]
fn test_regex_atom_anchoring() {
    // RegexAtom should automatically anchor to start
    let atom = RegexAtom::new(AtomKind::Number, r"\d+");
    let input = "abc 123";
    let cursor = Cursor::new(input);

    // Should NOT match 123 because it's not at the start
    assert!(atom.parse(cursor).is_none());
}

#[test]
fn test_regex_atom_empty_match() {
    // A regex that can match empty string should return None to avoid infinite loops
    let atom = RegexAtom::new(AtomKind::Other("empty".into()), r"a*");
    let input = "bcd"; // 'a*' matches empty string at start
    let cursor = Cursor::new(input);

    // Should return None because length is 0
    assert!(atom.parse(cursor).is_none());
}

#[test]
fn test_keyword_atom_basic() {
    let atom = KeywordAtom::new(&["if", "else", "while"]);
    let input = "while condition";
    let cursor = Cursor::new(input);

    let (token, next) = atom.parse(cursor).expect("Should match keyword");
    assert!(matches!(token.kind, AtomKind::Identifier(_)));
    assert_eq!(token.text, "while");
    assert_eq!(next.rest, " condition");
}

#[test]
fn test_keyword_atom_longest_match() {
    // "integer" should match "integer", not "int"
    let atom = KeywordAtom::new(&["int", "integer"]);
    let input = "integer x";
    let cursor = Cursor::new(input);

    let (token, next) = atom.parse(cursor).expect("Should match keyword");
    assert!(matches!(token.kind, AtomKind::Identifier(_)));
    assert_eq!(token.text, "integer");
    assert_eq!(next.rest, " x");
}

#[test]
fn test_keyword_atom_prefix_behavior() {
    // This documents the current behavior: KeywordAtom matches prefixes.
    // If we have "int" and input is "integer", it WILL match "int" if "integer" is not in the list.
    // This is why order of Atoms in the Language definition matters (Keywords vs Identifiers).
    let atom = KeywordAtom::new(&["int"]);
    let input = "integer";
    let cursor = Cursor::new(input);

    let (token, next) = atom.parse(cursor).expect("Should match prefix");
    assert_eq!(token.text, "int");
    assert_eq!(next.rest, "eger");
}
