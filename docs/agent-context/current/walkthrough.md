# Walkthrough

(This file will be populated as work progresses in the current phase.)

## Shape Algebra Implementation

We have implemented the core Shape Algebra in `src/shape.rs`, providing a set of composable primitives for matching patterns in a `TokenStream`.

### Key Decisions

1.  **Token Tree Model**: The algebra operates on `TokenTree` nodes, not a flat list of tokens. This requires specific combinators like `enter` to navigate into delimited groups.
2.  **Whitespace Handling**:
    - **Implicit Skipping**: The `term` and `enter` primitives automatically skip leading whitespace. This means `seq(a, b)` implicitly skips whitespace between `a` and `b`.
    - **Explicit Adjacency**: The `adjacent` combinator explicitly checks for the _absence_ of whitespace between two shapes, enabling "tight" binding (e.g., `a.b` vs `a . b`).
3.  **Matcher Trait**: We introduced a `Matcher` trait to abstract over different ways of matching a single token (by kind, by text, by delimiter type).
4.  **Derived Combinators**: Complex patterns like `separated` (lists) and `opt` (optional) are defined in terms of the core primitives, keeping the core API small and orthogonal.

### Implemented Primitives

- `term(matcher)`: Matches a single token/tree.
- `seq(a, b)`: Sequence.
- `choice(a, b)`: Ordered choice.
- `rep(a)`: Repetition (0 or more).
- `enter(delimiter, inner)`: Matches inside a delimited group.
- `adjacent(a, b)`: Sequence with no intervening whitespace.
- `empty()`: Always succeeds.
- `end()`: Succeeds at end of stream.

### Implemented Derived Combinators

- `opt(a)`
- `separated(item, sep)`
- `joined(a)`

## Variable Binding & Atomic Lexer

We implemented the mechanism for distinguishing between variable bindings and references during the atomic lexing phase.

### Key Decisions

1.  **VariableRole Enum**: We added `VariableRole` (Binding, Reference, None) to `AtomKind::Identifier`. This allows the parser to know the role of an identifier without needing a separate pass or complex context lookup during parsing.
2.  **VariableRules Trait**: We defined a `VariableRules` trait that allows languages to specify how to classify identifiers based on local context (e.g., "bind after `let`").
3.  **Atomic Lexer**: We implemented a recursive descent lexer in `src/lexer.rs` that:
    - Produces a tree of `TokenTree`s (handling delimiters).
    - Applies `VariableRules` on the fly, tracking the previous token to determine context.
    - Handles whitespace and comments (via `Atom` implementations).

### Implementation Details

- `src/atom.rs`: Updated `AtomKind::Identifier` to carry `VariableRole`.
- `src/language.rs`: Added `VariableRules` trait and `PatternVariableRules` implementation.
- `src/lexer.rs`: Implemented `lex` function and `lex_group` helper.
- `src/mock.rs`: Updated `MockLanguage` to support testing of binding rules.

## Macro Expansion Loop

We implemented the core parsing loop that handles macro expansion and operator precedence.

### Key Decisions

1.  **Expression Continuation Model**: The parser uses a loop that first parses a "head" term (literal, identifier, or prefix macro) and then repeatedly checks for "continuation" macros (infix/postfix operators) based on precedence.
2.  **MatchContext Trait**: To allow the `Shape` algebra to recurse back into the expression parser (e.g., for parsing the RHS of an operator), we introduced a `MatchContext` trait. The `Parser` implements this trait, exposing `parse_expression`.
3.  **Expr Shape**: We added a new shape primitive `expr(precedence)` that delegates to `MatchContext::parse_expression`. This allows macro signatures to declare that they expect an expression as an argument.
4.  **Macro Trait Updates**: We updated the `Macro` trait to include `precedence`, `associativity`, and `is_operator` flags to support the expression parsing loop.

### Implementation Details

- `src/shape.rs`: Added `MatchContext` trait, `Precedence` struct, `Associativity` enum, and `Expr` shape. Updated all shapes to pass `MatchContext`.
- `src/macro.rs`: Updated `Macro` trait to use `Precedence` and `Associativity`.
- `src/parser.rs`: Implemented the `Parser` struct with the `parse_expression` loop, handling prefix and infix macros.
- `src/lib.rs`: Exported `Parser`.

## Error Recovery

We implemented a "forgiving" parsing mechanism using the `recover` combinator.

### Key Decisions

1.  **Error TokenTree**: We added `TokenTree::Error(String)` to represent a part of the syntax tree that failed to parse but was recovered from.
2.  **Recover Shape**: We introduced `recover(shape, terminator)` which attempts to match `shape`. If it fails, it skips tokens until it finds a token matching `terminator` (or EOF), and returns a `TokenTree::Error` containing the skipped tokens' info. This allows the parser to continue parsing subsequent statements even after a syntax error.

### Implementation Details

- `src/token.rs`: Added `TokenTree::Error`.
- `src/shape.rs`: Added `Recover` struct and `recover` function.
- `src/parser.rs`: Added unit test `test_recover` verifying that the parser can skip malformed input and stop at a delimiter.
