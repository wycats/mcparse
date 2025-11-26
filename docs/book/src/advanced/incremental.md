# Incremental Parsing

As codebases grow, re-parsing the entire file on every keystroke becomes prohibitively expensive. **Incremental Parsing** solves this by reusing parts of the syntax tree that haven't changed.

McParse implements a strategy inspired by Roslyn (C#) and Rowan (Rust-analyzer), known as **Red/Green Trees**.

## The Red/Green Tree Model

The core idea is to split the syntax tree into two layers:

1.  **Green Tree (Internal, Immutable, Relative)**:

    - Nodes store their **width** (length in characters) but _not_ their absolute offset.
    - Nodes are **immutable**.
    - Because they don't know their position, identical nodes can be **shared** (structural sharing).
    - This is the "ground truth" of the parse structure.

2.  **Red Tree (External, Transient, Absolute)**:
    - A "cursor" or wrapper around a Green Node.
    - Stores the **absolute offset** of the node.
    - Created on-demand when traversing the tree.
    - Allows you to ask "what token is at offset X?".

### Data Structures

In `mcparse`, the Green Tree is defined as:

```rust
pub enum GreenTree {
    Token(GreenToken),
    Delimited(Box<GreenDelimited>),
    Group(Vec<GreenTree>),
    Empty,
}

pub struct GreenToken {
    pub kind: AtomKind,
    pub text: String,
    // width is text.len()
}
```

The Red Node is a transient wrapper:

```rust
pub struct RedNode<'a> {
    pub green: &'a GreenTree,
    pub offset: usize,
}
```

## Incremental Re-lexing

When a user edits the text, we want to update the Green Tree without re-parsing everything. McParse uses a **Conservative Re-lexing** strategy.

### The Algorithm

1.  **Locate the Edit**: Given a `TextEdit` (start, end, new text), we find the deepest `Delimited` node (like `{ ... }` or `( ... )`) that _fully contains_ the edit.
2.  **Isolate**: We extract the text of that node's content.
3.  **Apply & Re-lex**: We apply the edit to that isolated text and re-run the lexer on just that small chunk.
4.  **Stitch**: If re-lexing succeeds, we create a new `GreenTree` node for that block, reusing the old children that weren't touched.
5.  **Propagate**: We rebuild the path to the root, creating new parent nodes that point to the new child. Unchanged siblings are shared by reference (cheaply cloned).

### The "Bubble Up" Strategy

If an edit cannot be handled by a child node (e.g., because it deletes a closing brace `}`), the failure **bubbles up** to the parent.

1.  **Child Failure**: The child node reports that it cannot contain the edit.
2.  **Parent Recovery**: The parent node catches this failure and attempts to re-lex _its_ own content (which includes the broken child).
3.  **Root Fallback**: This continues up the tree. In the worst case, the Root node (usually a `Group` representing the whole file) will re-lex the entire file.

This ensures that we always re-parse the **smallest possible scope** that can contain the change, preserving as much of the tree as possible.

### Limitations

If an edit touches a delimiter (e.g., deleting a closing brace `}`), the edit is not "contained" within the block. In this case, the incremental step fails for that block, and we bubble up to the parent.

## Example

Here is how you might use the incremental API:

```rust
use mcparse::incremental::{GreenTree, TextEdit, apply_edit};

// 1. Initial Parse
let tokens = lex(initial_text, &lang);
let root = GreenTree::from_token_tree(tokens);

// 2. Define an Edit (e.g., changing a number inside a block)
let edit = TextEdit {
    start: 21,
    end: 22,
    new_text: "3".to_string(),
};

// 3. Apply Edit
// This automatically attempts incremental re-lexing,
// and falls back to a full re-parse if necessary.
let new_root = apply_edit(&root, &edit, &lang);

println!("Updated text: {}", new_root.text());
```
