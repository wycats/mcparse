# Phase 9: Scoping & Completion (Walkthrough)

## Goal

Enhance the developer experience by implementing intelligent Tab Completion for variables in the REPL.

## Changes

### 1. Refactored `VariableRules`

- Confirmed that `VariableRules` has been fully replaced by `BindingPass` and `ReferencePass`.
- Updated `DESIGN.md` to reflect the new architecture.

### 2. Implemented `Language::complete`

- Added `complete` method to `Language` trait.
- Implemented `find_completions` helper in `src/completion.rs` that uses `BindingPass` to populate `ScopeStack` up to the cursor position.
- Updated `BindingPass` trait to include `collect_scope_at` method for partial scope collection.
- Implemented `collect_scope_at` for `SimpleBindingPass`.

### 3. Unclosed Delimiter Handling

- Updated `TokenTree::Delimited` to store `is_closed` boolean.
- Updated `lexer` to populate this flag.
- Updated `collect_scope_at` to correctly handle unclosed delimiters (treating the cursor as "inside" if the group is unclosed and extends to EOF).
- Verified with tests in `src/completion.rs`.

### 4. Updated REPL

- Updated `examples/repl.rs` to use `Language::complete` for variable completion.
- Merged variable completions with shape-based completions (keywords).
- Updated `MiniScriptLang` to use `SimpleBindingPass` and `SimpleReferencePass`.

## Key Decisions

- **Explicit `is_closed` flag**: We added a boolean to `TokenTree::Delimited` to distinguish between closed groups (where the cursor must be strictly inside) and unclosed groups (where the cursor at EOF is considered inside). This is crucial for completion while typing.
- **`collect_scope_at`**: We added a method to `BindingPass` to allow "pausing" the binding analysis at a specific offset. This avoids the need to re-implement the binding logic just for completion.

```
