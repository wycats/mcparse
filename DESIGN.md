McParse is a parsing library designed to support flexible languages based on simple and composable primitives.

McParse's grammar is based on macros: every grammar construct is defined as a macro that supports composition and reuse. This allows users to define complex grammars by combining simpler ones, promoting modularity and maintainability.

McParse is written in Rust, leveraging Rust's strong type system and performance characteristics to provide efficient parsing capabilities. It's also exposed as a wasm library with TypeScript bindings, so you can use it in any language that supports WebAssembly, including client-side JavaScript.

> McParse uses async Rust for its token streams, allowing it to handle large inputs and complex grammars without blocking the main thread. This makes it suitable for use in web applications and other environments where responsiveness is critical.

## Invariants

All McParse grammars support the following invariants:

- Forgiving Parsing: McParse parsers are designed to handle incomplete or malformed input gracefully, providing meaningful error messages and recovery options.
- Syntax Highlighting:

## Goals

- **Lexical Core**: The core of a McParse grammar is its shapes, which define the structure of valid input, macro positions, which specify where macros can be applied within the input, and delimiters, which define the boundaries of syntactic constructs and define its token tree structure.
- **Shapes**: A McParse grammar specifies _atoms_ that are used to lexically analyze the input into a token stream, and _shapes_ that define the valid arrangements of these atoms.
- **Macros and Signatures**: Macros in McParse are defined with signatures that specify the expected input structure and behavior in terms of grammar shapes. Macros can include their own shapes, but not their own atoms.
- **Incremental Parsing**: McParse supports incremental parsing, allowing for efficient re-parsing of modified input without needing to re-parse the entire source. This also includes efficient representations of the source locations of tokens and token trees so that only the affected parts of the parse tree need to be updated.
- **Source Locations**: McParse tracks source locations for all parsed elements, enabling precise error reporting and tooling support.
- **Tab Completion**: McParse provides built-in support for tab completion, allowing users to easily extend and customize completion behavior. Tab completion is based on the specified signature of the macro where the cursor is located.
- **Syntax Highlighting**: McParse grammars inherently support syntax highlighting by providing detailed token information during parsing. This highlighting is macro-sensitive, allowing for context-aware coloring and styling of code elements even while the user is still finishing the full macro definition.
- **Macro Definition Phase**: McParse includes a dedicated phase for defining and registering macros before parsing begins, ensuring that all macros are available for use during the parsing process. The first lexing phase can include macro imports if a language wants to support that, but macros must be defined as part of the flat lexical pass before macro expansion begins.
- **Macros**: There is no single "lexing" pass over the entire input. Instead, lexing is performed on-demand by macros as they parse the input.
- **Token Tree Structure**: Represent source code as a tree of tokens, allowing for hierarchical parsing and manipulation.

## Parsing Process

The parsing process in McParse involves several key steps:

1. **Language Selection**: Before any atoms are processed, the parser is initialized with a specific language grammar, which defines the available macros, shapes, and atoms. Language grammars must _also_ specify rules for identifying variable bindings and references that are globally recognized. This can happen by passing a language to the parser, or by preprocessing the input to identify language selection constructs _before_ atom parsing begins.
2. **Hygiene**: Since macros cannot expand into variable bindings or references that are visible to the surface code using the macro, variable bindings and references can be identified and symbolized during the _atomic lexing_ phase. If a macro expands into code that includes variable bindings or references, those bindings and references are given new, unique symbols based on their _source_ location (i.e. the macro definition). This gives us hygiene without needing complicated renaming schemes during macro expansion or a Ph.D in macrology.
3. **Atomic Lexing**: The input is first processed to identify atoms (including delimiters), creating a flat token stream.
4. **Variable Assignment**: During atomic lexing, variable bindings and references are identified and symbolized based on the rules defined in the language grammar. After this step, all variable bindings and references in the token stream have unique symbols that identify them, and they are represented by special atom types that distinguish them from regular identifiers.
5. **Macro Imports**: As the parser processes the token stream, it adds specified macros to the current macro environment based upon macro import constructs found in the input. These import constructs are specified as part of the language grammar, but they must appear textually before any macros they import are used.
6. **Macro Expansion**: The parsing process is a macro expansion loop, where the parser repeatedly identifies and expands macros in the token stream until no macros remain. Delimiters are used to _bound_ macro expansions: if no macro name is found in a macro position, or if the macro's expected input shape cannot be matched, the parser processes the remaining shapes inside the current delimiter as atoms in an error state (i.e. until the closing delimiter for the current shape is found). Nested delimited regions are treated as tokens for this purpose, and when the parser is processing error tokens, it recursively processes all nested delimited regions as trees of atoms.
7. **Variable bindings and references**: A language grammar is required to define how variable bindings and references are represented in a macro-insensitive way.
8. **Error Handling**: Throughout the parsing process, McParse tracks errors and provides mechanisms for recovery and reporting, ensuring that users receive meaningful feedback on malformed input. Macros can also define custom error handling behavior as part of their signatures.
9. **Incremental Updates**: When the input is modified, McParse efficiently re-parses only the affected parts of the token tree, leveraging source location tracking to minimize processing time.
10. **Tab Completion**: During parsing, McParse identifies the cursor position and provides context-aware tab completion suggestions based on the current macro signature and input shape. The current position of the cursor is treated as a special token that can appear anywhere in the input that is inserted when the user hits the tab key.

## Atoms

A McParse grammar defines a set of atoms that are used to lexically analyze the input into a token stream. Atoms can include:

- Identifiers
- Keywords
- Atomic Literals (e.g., numbers, strings)
- Operators

A McParse grammar also defines a set of paired delimiters that are used to group tokens into hierarchical structures.

### Shapes

Shapes define the valid arrangements of atoms and delimiters in the input. In general, it's preferable to define "core" constructs (e.g. decimal numbers, tagged string literals, etc.) as shapes rather than atoms, since this allows for greater flexibility and composability in the grammar.

For example, a McParse grammar might define an integer atom, but define a decimal number shape that consists of one or more integer atoms separated by a decimal point atom. This allows the decimal number shape to be reused in other shapes (e.g., floating-point literals, version numbers, etc.) without needing to redefine the integer atom, and also allows for syntaxes like `1.days` to be defined as a shape that consists of a decimal number shape followed by a `.` atom and an identifier atom.

### Whitespace and Adjacency

To support rich syntactic distinctions—such as differentiating `3.4` (decimal) from `3 . 4` (syntax error or different construct), or `hello-world` (identifier) from `hello - world` (subtraction)—McParse handles whitespace at the **Atomic Lexing** layer but enforces constraints at the **Shape** layer.

1.  **Whitespace as Atoms**: Whitespace is lexed into atoms, preserving it in the token stream.
2.  **Default Ignorance**: By default, shapes ignore whitespace atoms between their components. A shape defined as `A B` will match `AB`, `A B`, and `A   B`.
3.  **Adjacency Constraints**: Shapes can explicitly require **adjacency** between components. An adjacency constraint asserts that two components must be adjacent in the token stream with _no_ intervening whitespace atoms.

This design implies the following constraints on the parser:

- **Shape Prioritization**: The parser must generally attempt to match "tight" shapes (those with adjacency constraints) before "loose" shapes. For example, `hello-world` is checked against `HyphenatedIdentifier` before `Subtraction`.
- **Local Backtracking**: If a tight shape fails solely due to the presence of whitespace (e.g., `3 . 4` failing `DecimalNumber`), the parser must be able to backtrack and attempt to match looser shapes.
- **Differentiating Atoms**: The parser must be able to distinguish `3.4` (where `.` is part of a number shape) from `3.days` (where `.` is a method call operator). This is handled by the shape definitions: `DecimalNumber` requires `Integer` + `.` + `Integer`, while `MethodCall` might accept `Expression` + `.` + `Identifier`.

### Variable Bindings and References: Examples

While McParse requires language grammars to define how variable bindings and references are represented, that doesn't mean that every language has to use the same constructs. For example:

- In a Lisp-like language, variable bindings might be represented by `define` forms, while references are simply identifiers. Such a language grammar is _permitted_ to distinguish between identifiers that reference variables and those that do not based upon scoping rules defined by the token tree structure. As long as the grammar can identify which identifiers are variable references and which are not _without macro expansion_, it is valid.
- In a language like JavaScript, variable bindings are represented by constructs like `var`, `let`, and `const`, while references are identifiers. Since JavaScript largely defines variable visibility based on _block containment_, the logic of a JavaScript-like grammar largely fits into the contours of McParse's requirements.
- A language like Handlebars trivially satisfies McParse's requirements, since variable bindings are defined as `as |variableName|` and identifiers are variable references if they are nested inside a block that binds the variable with the same name.

In general, this is not as difficult as it might seem, since languages with lexical scope tend to bind variables in terms of block scoping, and references are simply identifiers that appear within a block that binds them (for a global definition of "within the block" that can include things like JavaScript's hoisting rules). Since McParse's token tree structure inherently represents block containment, this is a natural fit.

The upside is that it's possible to identify _all_ variable bindings and references during the atomic lexing phase, which means that it's possible to rename variables hygienically during macro expansion even if a macro isn't found or has an error in its input shape. Fundamentally, this means that once the user believes they have defined a variable binding, no part of the grammar can "undo" that binding by redefining that identifier just for its macro expansion.

This also means that it's possible to _syntax highlight_ variable bindings and references even while the user is still typing a macro that contains them, since the variable binding and reference information is available during the atomic lexing phase.

### Expressions and Operators

To handle expressions and operators without ambiguity, McParse introduces the concept of **Expression Continuation**.

1.  **Macro Lookup**: The parser first attempts to match a macro name at the current position.
2.  **Variable Shadowing**: If a variable binding with the same name exists in the current scope, it shadows the macro. This allows new keywords to be introduced without breaking existing code that uses them as variable names.
3.  **Expression Fallback**: If no macro is found, the parser assumes it is parsing an **Expression**.
4.  **Operators as Continuations**: An operator is defined as a macro that _continues_ an expression.
    - It appears _after_ a complete expression.
    - It consumes the operator token and then expects another expression (or specific shape) to follow.
    - This process repeats until an expression ends without a subsequent continuation token.

This model avoids complex precedence rules in the core parser. Instead, the parser produces a flat list of expressions and operators. The language grammar then defines a precedence hierarchy to group this list into a tree of binary expressions (e.g., `x + y * z` becomes `x + (y * z)`).

## Rust Interfaces and Types

## Rust Interfaces and Types

### Atom Signature

```rust
/// Defines a type of atom in the language (e.g., Identifier, Integer, StringLiteral).
/// Atoms are the leaf nodes of the token tree produced during atomic lexing.
pub trait Atom: Debug + Send + Sync {
    /// Returns the unique kind of this atom.
    fn kind(&self) -> AtomKind;

    /// Attempts to parse this atom from the given input cursor.
    /// Returns the parsed atom and the remaining input if successful.
    fn parse<'a>(&self, input: Cursor<'a>) -> ParseResult<'a, Token>;

    /// Provides highlighting information for this atom.
    fn highlight(&self, token: &Token, highlighter: &mut dyn Highlighter);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AtomKind {
    Whitespace,
    Identifier,
    Keyword(String),
    Literal,
    Operator,
    // Delimiters are handled separately to form the token tree structure
}

/// Defines a pair of delimiters that create a structural boundary in the token tree.
#[derive(Debug, Clone)]
pub struct Delimiter {
    pub kind: &'static str, // e.g., "Paren", "Brace"
    pub open: &'static str,
    pub close: &'static str,
}
```

### Shape Signature

```rust
/// Defines a syntactic shape that can be matched against the token stream.
/// Shapes are composed of Atoms and other Shapes.
pub trait Shape: Debug + Send + Sync {
    /// Attempts to match this shape against the token stream.
    /// Returns the matched token tree and the remaining stream if successful.
    fn match_shape<'a>(&self, stream: TokenStream<'a>) -> MatchResult<'a>;

    /// Returns the "tightness" of the shape for adjacency checks.
    /// e.g., does it require the previous token to be adjacent?
    fn adjacency(&self) -> AdjacencyConstraint {
        AdjacencyConstraint::None
    }

    /// Provides tab completion suggestions for this shape at the given cursor position.
    fn complete<'a>(&self, stream: TokenStream<'a>, cursor: SourceLocation) -> CompletionFuture<'a>;
}

#[derive(Debug, Clone, Copy)]
pub enum AdjacencyConstraint {
    None,
    Required, // Must be adjacent to the previous token (no whitespace)
    Forbidden, // Must NOT be adjacent to the previous token
}
```

### Shape Algebra

```rust
/// Combinators for composing shapes.
pub mod shapes {
    /// Matches a sequence of shapes: A followed by B.
    pub fn seq(a: impl Shape, b: impl Shape) -> impl Shape { ... }

    /// Matches either shape A or shape B (ordered choice).
    pub fn choice(a: impl Shape, b: impl Shape) -> impl Shape { ... }

    /// Matches shape A repeated N times.
    pub fn rep(a: impl Shape) -> impl Shape { ... }

    /// Matches shape A optionally.
    pub fn opt(a: impl Shape) -> impl Shape { ... }

    /// Enforces adjacency between A and B (no whitespace allowed).
    pub fn adjacent(a: impl Shape, b: impl Shape) -> impl Shape { ... }

    /// Matches shape A separated by shape B (e.g., comma-separated list).
    pub fn separated(item: impl Shape, separator: impl Shape) -> impl Shape { ... }
}
```

### Macro Signatures

```rust
/// Defines the interface for a macro in the language.
pub trait Macro: Debug + Send + Sync {
    /// The name of the macro (e.g., "def", "if", "+").
    fn name(&self) -> &str;

    /// The expected input shape for this macro.
    fn signature(&self) -> &dyn Shape;

    /// Expands the macro into a new token tree based on the matched input.
    fn expand(&self, input: TokenTree, context: &MacroContext) -> ExpansionResult;

    /// Defines if this macro acts as an operator (expression continuation).
    fn is_operator(&self) -> bool { false }

    /// If it is an operator, what is its precedence and associativity?
    fn precedence(&self) -> Option<Precedence> { None }
}
```

### Language Specification

```rust
/// The entry point for defining a language grammar.
pub trait Language: Debug + Send + Sync {
    /// The set of atoms defined by this language.
    fn atoms(&self) -> &[Box<dyn Atom>];

    /// The set of delimiters that define the token tree structure.
    fn delimiters(&self) -> &[Delimiter];

    /// The set of initial macros available in the global scope.
    fn macros(&self) -> &[Box<dyn Macro>];

    /// The pass that identifies variable bindings in the token tree.
    fn binding_pass(&self) -> &dyn BindingPass;

    /// The pass that resolves variable references in the token tree.
    fn reference_pass(&self) -> &dyn ReferencePass;
}
```

## Inspired By

- Racket Macros: The macro system in Racket provides a powerful way to define and compose language constructs, which McParse adopts for its grammar definitions.
- Nushell: Nushell's approach to parsing and handling incomplete input has influenced McParse's forgiving parsing design.
- Tree-sitter: The tree-based representation of source code in Tree-sitter has inspired McParse's token tree structure. McParse grammars inherently support syntax highlighting by providing detailed token information during parsing.
- Rust Macros: Rust's macro system, particularly `macro_rules!`, demonstrates the power of pattern matching and hygiene in a statically typed language. McParse aims to bring similar capabilities to dynamic language definitions.
- Incremental Parsing
