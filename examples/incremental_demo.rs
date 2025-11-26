use mcparse::{
    define_language,
    incremental::{GreenTree, TextEdit, incremental_relex, RelexResult, RedNode},
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
    println!("\nApplying Edit: {:?} -> {:?}", edit, edit.apply(initial_text));

    // 3. Incremental Re-lex
    match incremental_relex(&root, &edit, &lang) {
        RelexResult::Success(new_root) => {
            println!("Incremental Success!");
            println!("New Text: {}", new_root.text());
            assert_eq!(new_root.text(), edit.apply(initial_text));
        }
        RelexResult::Failed => {
            println!("Incremental Failed (Expected Success)");
        }
    }

    // 4. Apply Edit: Break the block (delete '}')
    // "let x = 1; { let y = 2; }"
    //                        ^ index 24
    let edit_break = TextEdit {
        start: 24,
        end: 25,
        new_text: "".to_string(),
    };
    println!("\nApplying Breaking Edit: {:?} -> {:?}", edit_break, edit_break.apply(initial_text));

    match incremental_relex(&root, &edit_break, &lang) {
        RelexResult::Success(_) => {
            println!("Incremental Success (Unexpected?)");
            // If we delete '}', the inner re-lex might succeed but produce different tokens.
            // But wait, if we delete '}', the edit is overlapping the delimiter?
            // My logic checks `edit.end <= content_end`.
            // content_end is `node_end - close_len`.
            // If we delete '}', `edit.start` is at `content_end`.
            // So it is NOT strictly inside content.
            // So it should fail to recurse, and fail at the block level.
        }
        RelexResult::Failed => {
            println!("Incremental Failed (Expected, as it touches delimiter)");
        }
    }

    // 5. Red Node Traversal
    let red_root = RedNode::new(&root, 0);
    println!("\nRed Node Traversal:");
    if let Some(node) = red_root.find_at_offset(21) {
        println!("Node at 21: {:?} (Offset: {})", node.green, node.offset);
        // Should be the number '2' (or '3' if we used the new tree, but we are using 'root' here)
        // Wait, 'root' is the initial tree.
        // "let x = 1; { let y = 2; }"
        //                       ^ 21 is '2'
        if let GreenTree::Token(t) = node.green {
            assert_eq!(t.text, "2");
            println!("Found expected token: '2'");
        } else {
            println!("Found unexpected node type");
        }
    }
}
