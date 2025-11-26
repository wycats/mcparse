use mcparse::{define_language, lexer::lex, scoping::scope_tokens, token::TokenTree};

// --- Language ---

define_language! {
    struct ScopingLang;
    atoms = [
        atom Whitespace = regex r"\s+",
        atom Operator = "=",
        atom Operator = ";",
        atom Identifier = regex r"[a-zA-Z_][a-zA-Z0-9_]*",
        atom Number = regex r"\d+",
    ];
    delimiters = [
        delimiter "brace" = "{", "}",
    ];
    binding_pass = simple("let");
    reference_pass = simple;
}

fn print_tree(tree: &TokenTree, indent: usize) {
    let pad = "  ".repeat(indent);
    match tree {
        TokenTree::Token(t) => {
            let binding_info = match t.binding {
                Some(id) => format!(" [Binding: {:?}]", id),
                None => "".to_string(),
            };
            println!("{}{:?} '{}'{}", pad, t.kind, t.text, binding_info);
        }
        TokenTree::Delimited(d, children, _, _) => {
            println!("{}Delimited({})", pad, d.kind);
            for child in children {
                print_tree(child, indent + 1);
            }
        }
        TokenTree::Group(children) => {
            println!("{}Group", pad);
            for child in children {
                print_tree(child, indent + 1);
            }
        }
        TokenTree::Error(msg) => println!("{}Error: {}", pad, msg),
        TokenTree::Empty => println!("{}Empty", pad),
    }
}

fn main() {
    let lang = ScopingLang::new();

    // Test Case 1: Basic Binding and Reference
    println!("--- Test Case 1: Basic ---");
    let input = "let x = 1; x";
    let mut trees = lex(input, &lang);
    scope_tokens(&mut trees, &lang);
    for tree in &trees {
        print_tree(tree, 0);
    }

    // Test Case 2: Shadowing
    println!("\n--- Test Case 2: Shadowing ---");
    let input = "let x = 1; { let x = 2; x } x";
    let mut trees = lex(input, &lang);
    scope_tokens(&mut trees, &lang);
    for tree in &trees {
        print_tree(tree, 0);
    }

    // Test Case 3: Reference before definition (should be None)
    println!("\n--- Test Case 3: Hoisting Check (Should fail) ---");
    let input = "x; let x = 1;";
    let mut trees = lex(input, &lang);
    scope_tokens(&mut trees, &lang);
    for tree in &trees {
        print_tree(tree, 0);
    }
}
