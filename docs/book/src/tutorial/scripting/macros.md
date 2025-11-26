# Macros & Hygiene

In many parser toolkits, "macros" are an afterthought or a special feature. In McParse, they are **fundamental**.

Even standard language constructs like `let` statements or `if` expressions can be implemented as macros. This makes the parser extremely extensible.

## The `Macro` Trait

A macro in McParse is a struct that implements the `Macro` trait. It has three main responsibilities:

1.  **Name**: The keyword that triggers the macro (e.g., "let").
2.  **Signature**: A `Shape` that describes what arguments the macro accepts.
3.  **Expansion**: A function that transforms the arguments into a new `TokenTree`.

### Example: The `let` Macro

Let's implement `let <var> = <expr>`.

```rust
use mcparse::{
    r#macro::{Macro, ExpansionResult, MacroContext},
    shape::{Shape, seq, term, Matcher},
    token::{TokenTree},
    AtomKind
};

#[derive(Debug)]
struct AnyIdentifier;
impl Matcher for AnyIdentifier {
    fn matches(&self, tree: &TokenTree) -> bool {
        if let TokenTree::Token(t) = tree {
            matches!(t.kind, AtomKind::Identifier)
        } else {
            false
        }
    }
    fn describe(&self) -> String { "identifier".into() }
}

#[derive(Debug)]
struct LetMacro {
    signature: Box<dyn Shape>,
}

impl LetMacro {
    fn new() -> Self {
        // Signature: <identifier> "=" <number>
        // Note: We are simplifying <expr> to just <number> for now.
        let shape = seq(term(AnyIdentifier), seq(term("="), term(AtomKind::Number)));
        Self {
            signature: Box::new(shape),
        }
    }
}

impl Macro for LetMacro {
    // The keyword that triggers this macro.
    fn name(&self) -> &str {
        "let"
    }

    // The shape of the arguments that follow the keyword.
    fn signature(&self) -> &dyn Shape {
        &*self.signature
    }

    // The expansion logic.
    fn expand(
        &self,
        args: TokenTree,
        _lhs: Option<TokenTree>,
        _context: &MacroContext,
    ) -> ExpansionResult {
        // For a compiler, you would generate AST nodes here.
        // For now, we just return the parsed arguments as-is.
        ExpansionResult::Ok(args)
    }
}
```

## Hygiene and Scoping

One of the hardest parts of parsing programming languages is **Hygiene**: knowing which variable refers to which declaration.

McParse solves this with a **Scoping Pass**. After the lexing phase, we run a pass to classify identifiers as "Bindings" (declarations) or "References" (usages) and link them together.

We do this by implementing the `BindingPass` and `ReferencePass` traits on our `Language`.

```rust
use mcparse::{
    scoping::{BindingPass, ReferencePass, ScopeStack, NoOpBindingPass, NoOpReferencePass},
    token::{TokenTree},
    AtomKind
};

// For this tutorial, we'll use the NoOp passes, which do nothing.
// In a real language, you would implement logic to track scopes and bindings.
```

### Why is this cool?

By classifying variables _before parsing_, we get:

1.  **Syntax Highlighting**: We can color declarations differently from usages immediately.
2.  **Robustness**: The parser knows `x` is a binding in `let x = ...` without needing to fully parse the statement.

> **Note**: Because this happens before macro expansion, macros cannot dynamically introduce bindings based on their expansion in a way that confuses the parser.

## Registering Macros

Finally, we add the macro and scoping passes to our `Language` definition.

```rust
# use mcparse::{language::{Language, Delimiter}, r#macro::Macro, atom::Atom, scoping::{BindingPass, ReferencePass, NoOpBindingPass, NoOpReferencePass}};
# #[derive(Debug)] struct LetMacro;
# impl Macro for LetMacro { fn name(&self) -> &str { "" } fn signature(&self) -> &dyn mcparse::Shape { todo!() } fn expand(&self, _: mcparse::TokenTree, _: Option<mcparse::TokenTree>, _: &mcparse::MacroContext) -> mcparse::ExpansionResult { todo!() } }
# #[derive(Debug)] struct MiniScriptLang { macros: Vec<Box<dyn Macro>> }
impl Language for MiniScriptLang {
    // ... atoms and delimiters ...
    fn atoms(&self) -> &[Box<dyn Atom>] { &[] }
    fn delimiters(&self) -> &[Delimiter] { &[] }
    fn macros(&self) -> &[Box<dyn Macro>] {
        &self.macros // Contains Box::new(LetMacro::new())
    }
    fn binding_pass(&self) -> &dyn BindingPass {
        &NoOpBindingPass
    }
    fn reference_pass(&self) -> &dyn ReferencePass {
        &NoOpReferencePass
    }
}
```
