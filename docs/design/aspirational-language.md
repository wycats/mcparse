# Aspirational Language Definition: "JsonPlus"

This is a sketch of how a user might define a simple JSON-like language with comments and a few extra features using the McParse Rust API.

```rust
use mcparse::prelude::*;

// 1. Define Atoms
struct JsonAtoms;
impl AtomSet for JsonAtoms {
    fn define(&self, ctx: &mut AtomContext) {
        ctx.define_atom("Whitespace", atoms::whitespace());
        ctx.define_atom("String", atoms::quoted_string('"'));
        ctx.define_atom("Number", atoms::decimal_number());
        ctx.define_atom("Boolean", atoms::keyword("true") | atoms::keyword("false"));
        ctx.define_atom("Null", atoms::keyword("null"));
        ctx.define_atom("Comment", atoms::line_comment("//"));
        
        // Delimiters for objects and arrays
        ctx.define_delimiter("Brace", "{", "}");
        ctx.define_delimiter("Bracket", "[", "]");
    }
}

// 2. Define Shapes (Reusable patterns)
fn json_value() -> impl Shape {
    shapes::choice((
        shapes::atom("String"),
        shapes::atom("Number"),
        shapes::atom("Boolean"),
        shapes::atom("Null"),
        object_shape(),
        array_shape(),
    ))
}

fn object_pair() -> impl Shape {
    shapes::seq((
        shapes::atom("String"),
        shapes::token(":"),
        json_value()
    ))
}

fn object_shape() -> impl Shape {
    shapes::delimited("Brace", 
        shapes::separated(object_pair(), shapes::token(","))
    )
}

fn array_shape() -> impl Shape {
    shapes::delimited("Bracket",
        shapes::separated(json_value(), shapes::token(","))
    )
}

// 3. Define Macros (Top-level constructs or special forms)
// In a pure data language like JSON, everything might just be a shape.
// But let's add a "macro" for a special `import` directive.

struct ImportMacro;
impl Macro for ImportMacro {
    fn name(&self) -> &str { "import" }
    
    fn signature(&self) -> &dyn Shape {
        // Expects: import "filename" as identifier;
        &shapes::seq((
            shapes::atom("String"), // filename
            shapes::keyword("as"),
            shapes::atom("Identifier"), // binding
            shapes::token(";")
        ))
    }

    fn expand(&self, input: TokenTree, ctx: &MacroContext) -> ExpansionResult {
        // Implementation of import logic...
        ExpansionResult::Ok(TokenTree::empty()) 
    }
}

// 4. Define the Language
struct JsonPlus;
impl Language for JsonPlus {
    fn atoms(&self) -> &[Box<dyn Atom>] {
        // Return the set of atoms
        &[Box::new(JsonAtoms)]
    }

    fn macros(&self) -> &[Box<dyn Macro>] {
        &[Box::new(ImportMacro)]
    }

    fn root_shape(&self) -> &dyn Shape {
        // The top level is a sequence of values (or just one value)
        &json_value()
    }
    
    fn variable_rules(&self) -> &dyn VariableRules {
        // Simple rules: "Identifier" atoms in specific positions are bindings
        &SimpleVariableRules::default()
    }
}
```

## Key Takeaways from this Example

1.  **Composition**: Shapes are built by composing smaller shapes (`seq`, `choice`, `separated`).
2.  **Recursion**: `json_value` refers to `object_shape`, which refers back to `json_value`. The Rust API needs to support this (likely via `Lazy` or `Box<dyn Shape>`).
3.  **Macros vs Shapes**: `import` is a macro because it might do something special (like load another file) or introduce a binding. Standard JSON objects are just shapes.
4.  **Delimiters**: Braces and Brackets are defined as delimiters, which means the parser handles matching them and error recovery within them automatically.
