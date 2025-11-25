# Aspirational Language Definition: "MiniScript"

This language demonstrates McParse's macro system, variable binding, and expression parsing capabilities, going beyond the simple data structure of JSON.

```rust
use mcparse::prelude::*;

// 1. Define Atoms
struct MiniScriptAtoms;
impl AtomSet for MiniScriptAtoms {
    fn define(&self, ctx: &mut AtomContext) {
        ctx.define_atom("Whitespace", atoms::whitespace());
        ctx.define_atom("Identifier", atoms::identifier());
        ctx.define_atom("Number", atoms::decimal_number());
        ctx.define_atom("String", atoms::quoted_string('"'));
        ctx.define_atom("Operator", atoms::one_of(&["+", "-", "*", "/", "=", "==", "!="]));
        ctx.define_atom("Punctuation", atoms::one_of(&[";", ","]));

        // Delimiters
        ctx.define_delimiter("Brace", "{", "}");
        ctx.define_delimiter("Paren", "(", ")");
    }
}

// 2. Define Shapes
// Shapes are just matching rules. They don't produce custom AST nodes directly;
// they return the TokenTree structure that matched.

fn expression() -> impl Shape {
    // In a real implementation, this would likely hook into the
    // expression parsing logic (precedence, etc.), but for now,
    // let's say an expression is a sequence of atoms or parenthesized groups.
    shapes::rep(shapes::choice((
        shapes::atom("Number"),
        shapes::atom("String"),
        shapes::atom("Identifier"),
        shapes::delimited("Paren", expression()), // Recursive
    )))
}

fn block() -> impl Shape {
    shapes::delimited("Brace", shapes::rep(statement()))
}

fn statement() -> impl Shape {
    // A statement is either a macro invocation or an expression followed by a semicolon
    shapes::choice((
        // The parser's main loop handles macro dispatch, so "statement"
        // here effectively means "try to match a macro, otherwise..."
        // But in McParse, macros *are* the parsing loop.
        // So a "statement" shape is really just describing the structure
        // expected *inside* a block if it's not a macro.
        shapes::seq((expression(), shapes::token(";"))),
    ))
}

// 3. Define Macros

// `let` binding: let <name> = <expr>;
struct LetMacro;
impl Macro for LetMacro {
    fn name(&self) -> &str { "let" }

    fn signature(&self) -> &dyn Shape {
        &shapes::seq((
            shapes::atom("Identifier"), // The name to bind
            shapes::token("="),
            expression(),
            shapes::token(";")
        ))
    }

    fn expand(&self, input: TokenTree, ctx: &MacroContext) -> ExpansionResult {
        // Expansion logic would go here.
        // Crucially, the "Identifier" matched above is identified as a BINDING
        // by the VariableRules before we even get here.
        ExpansionResult::Ok(TokenTree::empty())
    }
}

// `if` control flow: if <expr> { ... } else { ... }
struct IfMacro;
impl Macro for IfMacro {
    fn name(&self) -> &str { "if" }

    fn signature(&self) -> &dyn Shape {
        &shapes::seq((
            expression(), // Condition
            block(),      // Then branch
            shapes::opt(shapes::seq((
                shapes::keyword("else"),
                block()   // Else branch
            )))
        ))
    }

    fn expand(&self, input: TokenTree, ctx: &MacroContext) -> ExpansionResult {
        ExpansionResult::Ok(TokenTree::empty())
    }
}

// `repeat` loop: repeat <count> { ... }
struct RepeatMacro;
impl Macro for RepeatMacro {
    fn name(&self) -> &str { "repeat" }

    fn signature(&self) -> &dyn Shape {
        &shapes::seq((
            shapes::atom("Number"),
            block()
        ))
    }

    fn expand(&self, input: TokenTree, ctx: &MacroContext) -> ExpansionResult {
        ExpansionResult::Ok(TokenTree::empty())
    }
}

// 4. Define the Language
struct MiniScript;
impl Language for MiniScript {
    fn atoms(&self) -> &[Box<dyn Atom>] {
        &[Box::new(MiniScriptAtoms)]
    }

    fn macros(&self) -> &[Box<dyn Macro>] {
        &[
            Box::new(LetMacro),
            Box::new(IfMacro),
            Box::new(RepeatMacro),
        ]
    }

    fn variable_rules(&self) -> &dyn VariableRules {
        // This is where the magic happens.
        // We define that an "Identifier" atom that appears immediately after a "let" keyword
        // is a Variable Binding.
        // All other "Identifier" atoms are Variable References.
        &PatternVariableRules::new()
            .bind_after_keyword("let")
    }
}

// 5. Example Program
/*
let x = 10;
let y = 20;

if x == 10 {
    let z = x + y;
    repeat 3 {
        let z = z + 1;
    };
} else {
    let z = 0;
}
*/
```
