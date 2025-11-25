# Design: Incremental Parsing

## Problem Statement

In an interactive editor, the user modifies the source code constantly. Re-parsing the entire file on every keystroke is inefficient for large files and can lead to loss of state (e.g., folding ranges, semantic highlighting stability).

`mcparse` needs a strategy to update its `TokenTree` structure incrementally.

## Architecture Constraints

1.  **Atomic Lexer**: Our lexer produces a `TokenTree` (nested structure), not a flat list of tokens. This makes incrementalism slightly more complex than a simple state-machine restart, but potentially more powerful because we can isolate changes to specific sub-trees (blocks).
2.  **Shape Algebra**: The parser consumes `TokenStream`s. If we can update the `TokenStream` in place (or produce a new one sharing structure with the old one), the parser can potentially skip re-parsing unchanged shapes.

## Proposed Strategy: Tree-Based Invalidation

### 1. The "Green Tree" Concept (Inspired by Roslyn/Rowan)

We can view our `TokenTree` as a "Green Tree" (immutable, structurally shared).

- **Edit Operation**: An edit consists of a `Range` (start, end) and `NewText`.
- **Locate**: We find the deepest node in the current `TokenTree` that fully contains the edit range.
- **Invalidate**: That node (and its parents) are invalidated.
- **Re-lex**: We re-lex only the text corresponding to that node.

### 2. Atomic Re-lexing

Because our lexer is "atomic" (it parses atoms and delimiters), we can re-lex a substring.

**Challenge**: Context sensitivity.
- If I type `/*` inside a block, it might comment out the rest of the file, invalidating everything after it.
- If I type `}` inside a block, it might close the block early.

**Solution**: Conservative Re-lexing.
1.  Identify the "governing delimiter" of the edit.
2.  Re-lex the entire content of that delimiter group.
3.  Compare the new tokens with the old tokens.
    - If the delimiters match (balanced), we can splice the new tree in.
    - If the delimiters are broken (e.g., user deleted a closing brace), the error propagates up, potentially requiring a re-lex of the parent.

### 3. Incremental Parsing (Shape Level)

Once we have a new `TokenStream`, we want to avoid re-running `match_shape` on everything.

- **Memoization**: We can memoize `match_shape(shape_id, start_token_index)`.
- **Identity**: If a `TokenTree` node is identical (by reference/ID) to the previous parse, and the Shape is the same, the result is the same.

## Data Structures

We may need to augment `TokenTree` to support this:

```rust
struct NodeId(u32);

struct TokenTree {
    id: NodeId, // For identity checks
    kind: TokenTreeKind,
    len: usize, // Byte length for range mapping
}
```

## Roadmap

1.  **Phase 1**: Re-lexing at the top-level only (naive).
2.  **Phase 2**: Re-lexing within the innermost `Delimited` group.
    - If the edit is strictly inside `{ ... }`, re-lex just that string.
    - If the result is a valid `TokenStream` (balanced), replace the group's children.
    - If not, escalate to parent.
3.  **Phase 3**: Memoized Parsing.

## Open Questions & Analysis

### Handling Macros

**Question**: If a macro definition changes, all usages need re-expansion. What are our options?

**Analysis**:
1.  **Global Invalidation (Simplest)**: If the set of defined macros changes (e.g., user edits a `macro_rules!` block), we invalidate the entire file's parse state. Macro definitions are relatively rare compared to macro usages.
2.  **Dependency Tracking (Advanced)**: We could track which nodes invoked which macros.
    - Map: `MacroName -> Vec<NodeId>` (Usages).
    - If `MacroName` changes, we invalidate only the specific `NodeId`s in that list.
    - **Challenge**: If the macro definition changes *how* it parses arguments, the "usage" node itself might change boundaries.

**Recommendation**: Start with **Global Invalidation** for macro *definitions*. For macro *usages* (editing the arguments passed to a macro), we can use the standard incremental re-lexing strategy (re-expand just that invocation).

### Text Range Mapping

**Question**: How do we map linear edit offsets to the tree structure efficiently?

**Prior Art (Roslyn/Rowan)**:
The standard solution is the **Red/Green Tree** split.

1.  **Green Tree (Internal, Storage)**:
    - Nodes are immutable and structurally shared.
    - Nodes store their **Length** (width in bytes), but *not* their absolute offset.
    - This allows a node representing `1 + 1` to be reused anywhere in the code without changing its internal data.

2.  **Red Tree (API, Cursor)**:
    - A transient wrapper around a Green Node.
    - Created on demand during traversal.
    - Stores: `Reference to Green Node`, `Parent Red Node`, and `Absolute Offset`.
    - The absolute offset is computed lazily as you traverse down from the root.

**Algorithm for Offset Mapping**:
To find the node at offset `X`:
1.  Start at the Root (Red Node, offset 0).
2.  Iterate through the children (Green Nodes).
3.  Keep a running `current_offset`.
4.  If `current_offset + child.len() > X`, then the target is inside this child.
5.  Construct the Red Node for that child (with `absolute_offset = current_offset`) and recurse.
6.  Complexity: `O(depth)` (logarithmic for balanced trees).

**Benefit: Efficient Range Queries**:
Yes, this approach means we can trivially ask for a range and it is computed efficiently on demand.
- **Green Nodes** don't know their position, so they can be shared.
- **Red Nodes** are created only when you traverse to them. As you traverse, you accumulate the offset.
- Once you hold a **Red Node**, asking for its absolute range is `O(1)` (it stores the start offset, and looks up the length from the Green Node).
- You never need to "recalculate" ranges for the whole tree after an edit; you just traverse the new path, and the offsets are naturally correct because they are derived from the lengths of the preceding siblings.

**Conclusion**: We should adopt the Red/Green tree model. Our current `TokenTree` is effectively a "Green Tree" (it doesn't store absolute offsets, only relative spans or lengths). We just need to formalize the "Red" cursor layer for traversal.
