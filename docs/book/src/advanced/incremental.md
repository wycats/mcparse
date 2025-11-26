# Incremental Parsing

McParse is designed to handle large files efficiently by only re-parsing what changed. It achieves this using a **Red/Green Tree** architecture, similar to Roslyn (C#) and rust-analyzer.

## The Data Structure

### Green Trees (Internal Structure)
-   **Immutable**: Once created, they never change.
-   **Position-Independent**: They don't know their absolute offset in the file. They only know their length.
-   **Structurally Shared**: If two functions are identical, they can point to the exact same Green Node in memory.

### Red Trees (Public API)
-   **Transient**: Created on demand when you traverse the tree.
-   **Context-Aware**: They know their parent and their absolute offset.
-   **Wrappers**: A Red Node is just a lightweight wrapper around a Green Node + an offset.

## How Incremental Updates Work

When you edit a file (e.g., insert a character):

1.  **Invalidation**: McParse finds the smallest node in the Green Tree that contains the edit.
2.  **Re-lexing**: It re-runs the Atomic Lexer *only* on the text of that node (plus the new character).
3.  **Re-parsing**: It attempts to re-parse that node using the original Shape.
4.  **Stitching**: If successful, it creates a new Green Node and "stitches" it back into the tree, reusing the unchanged siblings.

This means that typing inside a function body usually only triggers re-parsing of that single function, not the whole file.

