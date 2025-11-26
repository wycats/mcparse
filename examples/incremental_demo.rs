use mcparse::{
    define_language,
    incremental::{GreenTree, RedNode, TextEdit, apply_edit},
    lexer::lex,
};

define_language! {
    struct DemoLang;
    atoms = [
        atom Whitespace = regex r"\s+",
        atom Identifier = regex r"[a-zA-Z_]\w*",
        atom Number = regex r"\d+",
        atom Operator = "+",
        atom Operator = "=",
        atom Operator = ";",
    ];
    delimiters = [
        delimiter "brace" = "{", "}",
    ];
}

fn main() {
    let lang = DemoLang::new();
    let initial_text = "let x = 1; { let y = 2; }";
    println!("Initial: {}", initial_text);

    // 1. Initial Parse
    let tokens = lex(initial_text, &lang);
    // Wrap in a Group to act as Root
    let root = GreenTree::Group(tokens.iter().map(GreenTree::from_token_tree).collect());

    println!("Green Tree Width: {}", root.width());
    assert_eq!(root.width(), initial_text.len());

    // 2. Apply Edit: Change '2' to '3' inside the block
    // "let x = 1; { let y = 2; }"
    //                       ^ index 21
    let edit = TextEdit {
        start: 21,
        end: 22,
        new_text: "3".to_string(),
    };
    println!(
        "\nApplying Edit (Incremental): {:?} -> {:?}",
        edit,
        edit.apply(initial_text)
    );

    let new_root = apply_edit(&root, &edit, &lang);
    println!("New Text: {}", new_root.text());
    assert_eq!(new_root.text(), edit.apply(initial_text));

    // 3. Apply Edit: Break the block (delete '}')
    // "let x = 1; { let y = 2; }"
    //                        ^ index 24
    let edit_break = TextEdit {
        start: 24,
        end: 25,
        new_text: "".to_string(),
    };
    println!(
        "\nApplying Breaking Edit (Fallback): {:?} -> {:?}",
        edit_break,
        edit_break.apply(initial_text)
    );

    let broken_root = apply_edit(&root, &edit_break, &lang);
    println!("New Text: {}", broken_root.text());
    assert_eq!(broken_root.text(), edit_break.apply(initial_text));

    // 4. Red Node Traversal
    let red_root = RedNode::new(&root, 0);
    println!("\nRed Node Traversal:");
    if let Some(node) = red_root.find_at_offset(21) {
        println!("Node at 21: {:?} (Offset: {})", node.green, node.offset);
        if let GreenTree::Token(t) = node.green {
            assert_eq!(t.text, "2");
            println!("Found expected token: '2'");
        } else {
            println!("Found unexpected node type");
        }
    }

    // 5. Verify Bubble-Up Behavior
    // "let x = 1; { { let y = 2; } }"
    // We want to show that breaking the inner block is caught by the outer block,
    // NOT by the root.
    println!("\n--- Bubble Up Verification ---");
    let nested_text = "let x = 1; { { let y = 2; } }";
    let nested_tokens = lex(nested_text, &lang);
    let nested_root = GreenTree::Group(
        nested_tokens
            .iter()
            .map(GreenTree::from_token_tree)
            .collect(),
    );

    // Delete inner '}' at index 26
    // "let x = 1; { { let y = 2; } }"
    //                           ^ 26
    let edit_nested = TextEdit {
        start: 26,
        end: 27,
        new_text: "".to_string(),
    };

    // We expect this to be handled by incremental_relex (returning Success),
    // because the outer block '{ ... }' can contain the edit.
    // The Root (Group) shouldn't need to re-lex the "let x = 1;" part.

    use mcparse::incremental::incremental_relex;
    match incremental_relex(&nested_root, &edit_nested, &lang) {
        mcparse::incremental::RelexResult::Success(new_root) => {
            println!("Bubble Up Success: The outer block handled the broken inner block.");
            println!("New Text: {}", new_root.text());

            // Verify structural sharing?
            // The first child of root is "let", "x", "=", "1", ";".
            // These should be identical references (if we could check).
            // For now, we just trust the algorithm.
        }
        mcparse::incremental::RelexResult::Failed => {
            println!("Bubble Up Failed! (Unexpected)");
        }
    }
}
