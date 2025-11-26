# Phase 7: Incremental Parsing Implementation (Walkthrough)

## Goal

Implement the "Red/Green Tree" architecture and incremental re-lexing strategy to enable efficient updates to the parse tree when the source code changes.

## Changes

### 1. Green Tree Structure (`src/incremental.rs`)

We implemented the **Green Tree** data structure, which is immutable and position-independent.

- `GreenToken`: Stores `AtomKind` and text. Knows its width but not its offset.
- `GreenTree`: Recursive enum (`Token`, `Delimited`, `Group`, `Empty`).
- `GreenTree::from_token_tree`: Converts legacy `TokenTree` (with absolute offsets) to `GreenTree`.

### 2. Red Node Traversal (`src/incremental.rs`)

We implemented the **Red Node** cursor, which provides absolute offsets on demand.

- `RedNode`: Wraps a `GreenTree` reference and an `offset`.
- `RedNode::children()`: Lazily computes children with correct absolute offsets.
- `RedNode::find_at_offset(target)`: Efficiently locates the node at a specific position.

### 3. Incremental Re-lexing (`src/incremental.rs`)

We implemented `incremental_relex`, a function that attempts to apply a `TextEdit` by re-lexing only the smallest containing `Delimited` block.

- **Algorithm**:
    1.  Find the deepest `Delimited` node that fully contains the edit range.
    2.  Extract the text of its children.
    3.  Apply the edit to that text.
    4.  Re-lex the new text using the `Language`.
    5.  If successful, return a new `GreenTree` with the updated children, sharing the rest of the tree structure.
    6.  If the edit touches delimiters or cannot be isolated, return `RelexResult::Failed` (signaling a need for full re-parse).

### 4. Verification

- Created `examples/incremental_demo.rs`.
- Verified:
    - **Structural Sharing**: The root node width matches the text length.
    - **Incremental Update**: Editing inside a block (`{ let y = 2; }` -> `{ let y = 3; }`) successfully updates just that block.
    - **Failure Case**: Breaking a delimiter (deleting `}`) correctly fails the incremental step, as expected.
    - **Red Node Traversal**: Confirmed we can find tokens by absolute offset in the Green Tree.

## Key Decisions

- **Red/Green Split**: Adopted the standard Roslyn/Rowan model for structural sharing.
- **Conservative Re-lexing**: We only re-lex if the edit is strictly contained within a block's *content*. Touching delimiters forces a failure (and thus a parent re-lex or full re-parse) to ensure safety.
