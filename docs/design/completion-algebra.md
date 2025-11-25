# Completion Algebra

The Completion Algebra extends the Shape Algebra to support IDE-like tab completion. It allows a Shape to inspect the current cursor position and the token stream to suggest valid next tokens.

## Core Concepts

### `CompletionItem`

A single suggestion to be shown to the user.

```rust
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
}

pub enum CompletionKind {
    Keyword,
    Variable,
    Function,
    Field,
    // ...
}
```

### `Matcher::suggest`

The `Matcher` trait is extended to provide suggestions based on a partial match.

```rust
trait Matcher {
    fn matches(&self, tree: &TokenTree) -> bool;
    fn suggest(&self, current_token: &Token) -> Vec<CompletionItem>;
}
```

- If `Matcher` is a literal string (e.g., `"function"`), and the current token is `fun`, it suggests `"function"`.
- If `Matcher` is `AtomKind::Identifier`, it might delegate to a symbol table (via `MatchContext`) to suggest variables in scope.

### `Shape::complete`

The `Shape` trait is extended with a `complete` method.

```rust
fn complete<'a>(
    &self,
    stream: TokenStream<'a>,
    context: &mut dyn MatchContext,
    cursor: SourceLocation
) -> Vec<CompletionItem>;
```

## Completion Logic by Shape

### `Term<M>` (The Leaf)

1.  **Skip Whitespace**: Advance stream past whitespace.
2.  **Check Cursor**:
    - If the stream is empty but the cursor is at the end of the previous token (or EOF), suggest all valid matches for `M`.
    - If the stream has a token `T`:
      - If the cursor is **inside** or **touching** `T`, call `M.suggest(T)`.
      - If the cursor is **before** `T`, suggest all valid matches for `M` (insert mode).

### `Seq<A, B>` (The Chain)

1.  **Try Match A**: Attempt to parse `A`.
2.  **If A Matches**:
    - Check if the cursor falls within the span of tokens consumed by `A`.
    - **Inside A**: Return `A.complete(...)`.
    - **After A**: Advance the stream past `A` and return `B.complete(...)`.
3.  **If A Fails**:
    - Assume the user is currently typing `A`. Return `A.complete(...)`.

### `Choice<A, B>` (The Fork)

1.  **Try Match A**:
    - If `A` matches and the cursor is **inside** `A`, return `A.complete(...)`.
    - If `A` matches but the cursor is **after** `A`, this branch is "done". Return empty (let parent handle).
2.  **Try Match B**:
    - If `B` matches and the cursor is **inside** `B`, return `B.complete(...)`.
3.  **Both Fail**:
    - The user is likely at the start of the choice or typing something that matches neither yet.
    - Return **Union** of `A.complete(...)` and `B.complete(...)`.

### `Rep<A>` (The Loop)

1.  **Iterate**: Loop while `A` matches.
2.  **Check Cursor**:
    - If the cursor is **inside** the current instance of `A`, return `A.complete(...)`.
    - If the cursor is **after** the current instance, continue to next iteration.
3.  **End of Loop**:
    - If `A` fails to match (or EOF), we are at a potential new item. Return `A.complete(...)`.

### `Enter<S>` (The Scope)

1.  **Check Delimiter**: Ensure the next token is the open delimiter.
2.  **Check Cursor**:
    - If the cursor is **inside** the delimited group (between open and close), enter the group.
    - Create a new `TokenStream` from the group's content.
    - Return `S.complete(inner_stream, context, cursor)`.
