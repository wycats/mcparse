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
    // Helper: Resolve AtomKind
    (@atom_kind Identifier) => { $crate::atom::AtomKind::Identifier };
    (@atom_kind $kind:ident) => { $crate::atom::AtomKind::$kind };

    // Helper: Push atoms to vector
    (@atom_list_push $v:ident) => {};
    (@atom_list_push $v:ident,) => {};

    // Case: atom Kind = regex "pattern"
    (@atom_list_push $v:ident, atom $kind:ident = regex $regex:literal $(, $($rest:tt)*)?) => {
        $v.push(Box::new($crate::atoms::RegexAtom::new(
            $crate::define_language!(@atom_kind $kind),
            $regex
        )));
        $crate::define_language!(@atom_list_push $v, $($($rest)*)?)
    };

    // Case: atom Kind = "literal"
    (@atom_list_push $v:ident, atom $kind:ident = $literal:literal $(, $($rest:tt)*)?) => {
        $v.push(Box::new($crate::atoms::LiteralAtom::new(
            $crate::define_language!(@atom_kind $kind),
            $literal
        )));
        $crate::define_language!(@atom_list_push $v, $($($rest)*)?)
    };

    // Case: keywords [ "a", "b" ]
    (@atom_list_push $v:ident, keywords [ $($kw:literal),* $(,)? ] $(, $($rest:tt)*)?) => {
        $v.push(Box::new($crate::atoms::KeywordAtom::new(&[ $($kw),* ])));
        $crate::define_language!(@atom_list_push $v, $($($rest)*)?)
    };

    // Case: keyword "kw"
    (@atom_list_push $v:ident, keyword $kw:literal $(, $($rest:tt)*)?) => {
        $v.push(Box::new($crate::atoms::KeywordAtom::new(&[$kw])));
        $crate::define_language!(@atom_list_push $v, $($($rest)*)?)
    };

    // Case: expression (fallback)
    (@atom_list_push $v:ident, $e:expr $(, $($rest:tt)*)?) => {
        $v.push(Box::new($e));
        $crate::define_language!(@atom_list_push $v, $($($rest)*)?)
    };

    // Helper: Push delimiters to vector
    (@delimiter_list_push $v:ident) => {};
    (@delimiter_list_push $v:ident,) => {};

    // Case: delimiter "kind" = "open", "close"
    (@delimiter_list_push $v:ident, delimiter $kind:literal = $open:literal, $close:literal $(, $($rest:tt)*)?) => {
        $v.push($crate::language::Delimiter {
            kind: $kind,
            open: $open,
            close: $close,
        });
        $crate::define_language!(@delimiter_list_push $v, $($($rest)*)?)
    };

    // Case: expression (fallback)
    (@delimiter_list_push $v:ident, $e:expr $(, $($rest:tt)*)?) => {
        $v.push($e);
        $crate::define_language!(@delimiter_list_push $v, $($($rest)*)?)
    };

    // Helper for binding_pass default
    // (Removed old helpers as they are replaced by @parse_options)

    // --- Option Parsing (TT Muncher) ---

    // Case: macros = [ ... ];
    (@parse_options
        meta = $meta:tt, name = $name:ident, atoms = $atoms:tt, delimiters = $delimiters:tt,
        macros = $_old_macros:tt, binding_pass = $bp:tt, reference_pass = $rp:tt,
        input = [ macros = [ $($m:expr),* $(,)? ]; $($rest:tt)* ]
    ) => {
        $crate::define_language! { @parse_options
            meta = $meta, name = $name, atoms = $atoms, delimiters = $delimiters,
            macros = [ $($(Box::new($m)),*)? ],
            binding_pass = $bp, reference_pass = $rp,
            input = [ $($rest)* ]
        }
    };

    // Case: binding_pass = simple("kw");
    (@parse_options
        meta = $meta:tt, name = $name:ident, atoms = $atoms:tt, delimiters = $delimiters:tt,
        macros = $macros:tt, binding_pass = $_old:tt, reference_pass = $rp:tt,
        input = [ binding_pass = simple($kw:literal); $($rest:tt)* ]
    ) => {
        $crate::define_language! { @parse_options
            meta = $meta, name = $name, atoms = $atoms, delimiters = $delimiters,
            macros = $macros,
            binding_pass = { Box::new($crate::scoping::SimpleBindingPass::new($kw)) },
            reference_pass = $rp,
            input = [ $($rest)* ]
        }
    };

    // Case: binding_pass = expr;
    (@parse_options
        meta = $meta:tt, name = $name:ident, atoms = $atoms:tt, delimiters = $delimiters:tt,
        macros = $macros:tt, binding_pass = $_old:tt, reference_pass = $rp:tt,
        input = [ binding_pass = $e:expr; $($rest:tt)* ]
    ) => {
        $crate::define_language! { @parse_options
            meta = $meta, name = $name, atoms = $atoms, delimiters = $delimiters,
            macros = $macros,
            binding_pass = { Box::new($e) },
            reference_pass = $rp,
            input = [ $($rest)* ]
        }
    };

    // Case: reference_pass = simple;
    (@parse_options
        meta = $meta:tt, name = $name:ident, atoms = $atoms:tt, delimiters = $delimiters:tt,
        macros = $macros:tt, binding_pass = $bp:tt, reference_pass = $_old:tt,
        input = [ reference_pass = simple; $($rest:tt)* ]
    ) => {
        $crate::define_language! { @parse_options
            meta = $meta, name = $name, atoms = $atoms, delimiters = $delimiters,
            macros = $macros,
            binding_pass = $bp,
            reference_pass = { Box::new($crate::scoping::SimpleReferencePass) },
            input = [ $($rest)* ]
        }
    };

    // Case: reference_pass = expr;
    (@parse_options
        meta = $meta:tt, name = $name:ident, atoms = $atoms:tt, delimiters = $delimiters:tt,
        macros = $macros:tt, binding_pass = $bp:tt, reference_pass = $_old:tt,
        input = [ reference_pass = $e:expr; $($rest:tt)* ]
    ) => {
        $crate::define_language! { @parse_options
            meta = $meta, name = $name, atoms = $atoms, delimiters = $delimiters,
            macros = $macros,
            binding_pass = $bp,
            reference_pass = { Box::new($e) },
            input = [ $($rest)* ]
        }
    };

    // Case: Done (input empty)
    (@parse_options
        meta = [ $(#[$meta:meta])* ],
        name = $name:ident,
        atoms = [ $($atoms:tt)* ],
        delimiters = [ $($delimiters:tt)* ],
        macros = [ $($macros:expr),* ],
        binding_pass = { $binding_pass_impl:expr },
        reference_pass = { $reference_pass_impl:expr },
        input = []
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name {
            atoms: Vec<Box<dyn $crate::atom::Atom>>,
            delimiters: Vec<$crate::language::Delimiter>,
            macros: Vec<Box<dyn $crate::r#macro::Macro>>,
            binding_pass: Box<dyn $crate::scoping::BindingPass>,
            reference_pass: Box<dyn $crate::scoping::ReferencePass>,
        }

        impl $name {
            pub fn new() -> Self {
                let mut atoms: Vec<Box<dyn $crate::atom::Atom>> = Vec::new();
                $crate::define_language!(@atom_list_push atoms, $($atoms)*);

                let mut delimiters: Vec<$crate::language::Delimiter> = Vec::new();
                $crate::define_language!(@delimiter_list_push delimiters, $($delimiters)*);
                // Suppress unused_mut warning if delimiters is empty
                let _ = &mut delimiters;

                Self {
                    atoms,
                    delimiters,
                    macros: vec![ $($macros),* ],
                    binding_pass: $binding_pass_impl,
                    reference_pass: $reference_pass_impl,
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
            fn binding_pass(&self) -> &dyn $crate::scoping::BindingPass {
                self.binding_pass.as_ref()
            }
            fn reference_pass(&self) -> &dyn $crate::scoping::ReferencePass {
                self.reference_pass.as_ref()
            }
        }
    };

    // Main entry point
    (
        $(#[$meta:meta])*
        struct $name:ident;
        atoms = [ $($atoms:tt)* ];
        delimiters = [ $($delimiters:tt)* ];
        $($rest:tt)*
    ) => {
        $crate::define_language! { @parse_options
            meta = [ $(#[$meta])* ],
            name = $name,
            atoms = [ $($atoms)* ],
            delimiters = [ $($delimiters)* ],
            macros = [],
            binding_pass = { Box::new($crate::scoping::NoOpBindingPass) },
            reference_pass = { Box::new($crate::scoping::NoOpReferencePass) },
            input = [ $($rest)* ]
        }
    };
}
