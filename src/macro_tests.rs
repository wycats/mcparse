use crate::{atom::AtomKind, define_language, language::Language};

define_language! {
    struct TestLang;
    atoms = [
        atom Identifier = regex r"[a-zA-Z_]\w*",
        atom Number = regex r"\d+",
        atom Operator = "+",
        keyword "if",
        keywords [ "else", "while" ],
    ];
    delimiters = [
        delimiter "paren" = "(", ")",
    ];
}

#[test]
fn test_macro_expansion() {
    let lang = TestLang::new();
    let atoms = lang.atoms();

    assert_eq!(atoms.len(), 5);

    // Check kinds
    assert!(matches!(atoms[0].kind(), AtomKind::Identifier));
    assert!(matches!(atoms[1].kind(), AtomKind::Number));
    assert!(matches!(atoms[2].kind(), AtomKind::Operator));
    assert!(matches!(atoms[3].kind(), AtomKind::Identifier));
    assert!(matches!(atoms[4].kind(), AtomKind::Identifier));

    let delimiters = lang.delimiters();
    assert_eq!(delimiters.len(), 1);
    assert_eq!(delimiters[0].kind, "paren");
    assert_eq!(delimiters[0].open, "(");
    assert_eq!(delimiters[0].close, ")");
}
