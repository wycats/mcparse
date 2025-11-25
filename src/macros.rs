#[macro_export]
macro_rules! define_atom {
    (
        $(#[$meta:meta])*
        struct $name:ident;
        kind = $kind:expr;
        $(parse($input:ident) $parse_body:block)?
        $(highlight($token:ident, $highlighter:ident) $highlight_body:block)?
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl $crate::atom::Atom for $name {
            fn kind(&self) -> $crate::atom::AtomKind {
                $kind
            }

            fn parse<'a>(&self, input: $crate::token::Cursor<'a>) -> Option<($crate::token::Token, $crate::token::Cursor<'a>)> {
                $(
                    let $input = input;
                    $parse_body
                )?
            }

            fn highlight(&self, token: &$crate::token::Token, highlighter: &mut dyn $crate::highlighter::Highlighter) {
                $(
                    let $token = token;
                    let $highlighter = highlighter;
                    $highlight_body
                )?
            }
        }
    };
}

#[macro_export]
macro_rules! define_language {
    (
        $(#[$meta:meta])*
        struct $name:ident;
        atoms = [ $($atom:expr),* $(,)? ];
        delimiters = [ $($delimiter:expr),* $(,)? ];
        $(macros = [ $($macro:expr),* $(,)? ];)?
        $(variable_rules = $var_rules:expr;)?
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name {
            atoms: Vec<Box<dyn $crate::atom::Atom>>,
            delimiters: Vec<$crate::language::Delimiter>,
            macros: Vec<Box<dyn $crate::r#macro::Macro>>,
            variable_rules: Box<dyn $crate::language::VariableRules>,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    atoms: vec![ $(Box::new($atom)),* ],
                    delimiters: vec![ $($delimiter),* ],
                    macros: vec![ $($(Box::new($macro)),*)? ],
                    variable_rules: Box::new($($var_rules)?),
                }
            }
        }

        impl $crate::language::Language for $name {
            fn atoms(&self) -> &[Box<dyn $crate::atom::Atom>] {
                &self.atoms
            }
            fn delimiters(&self) -> &[$crate::language::Delimiter] {
                &self.delimiters
            }
            fn macros(&self) -> &[Box<dyn $crate::r#macro::Macro>] {
                &self.macros
            }
            fn variable_rules(&self) -> &dyn $crate::language::VariableRules {
                self.variable_rules.as_ref()
            }
        }
    };
}
