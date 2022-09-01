pub mod ast_shortcuts;

mod definition;
mod expression;
mod knowledge;
mod meta;
mod statement;

pub use definition::Definition;
pub use expression::{Expression, ExpressionInfixOpcode, ExpressionPrefixOpcode};
pub use knowledge::{MemoryKnowledge, TypeKnowledge, TypeReduction};
pub use meta::{FillMeta, Meta};
pub use statement::*;

#[derive(Clone)]
pub struct MainComponent {
    pub public_inputs: Vec<String>,
    pub initial_template_call: Expression,
}

impl MainComponent {
    pub fn new(public_inputs: Vec<String>, initial_template_call: Expression) -> Self {
        Self {
            public_inputs,
            initial_template_call,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Version {
    major: usize,
    minor: usize,
    patch: usize,
}

#[derive(Clone, Debug)]
pub enum Pragma {
    Version(Version),
    CustomGates,
    Empty,
}

impl Version {
    pub fn new(major: usize, minor: usize, patch: usize) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn major(&self) -> usize {
        self.major
    }

    pub fn minor(&self) -> usize {
        self.minor
    }

    pub fn patch(&self) -> usize {
        self.patch
    }
}

#[derive(Clone)]
pub struct AST {
    pub meta: Meta,
    pub compiler_version: Option<Version>,
    pub custom_gates: bool,
    pub custom_gates_declared: bool,
    pub includes: Vec<String>,
    pub definitions: Vec<Definition>,
    pub main_component: Option<MainComponent>,
}

impl AST {
    pub fn new(
        meta: Meta,
        pragmas: Vec<Pragma>,
        includes: Vec<String>,
        definitions: Vec<Definition>,
        main_component: Option<MainComponent>,
    ) -> AST {
        let mut custom_gates = None;
        let mut compiler_version = None;

        for p in pragmas {
            match p {
                Pragma::Version(ver) => match compiler_version {
                    Some(_) => panic!("multiple circom pragmas"),
                    None => compiler_version = Some(ver),
                },
                Pragma::CustomGates => match custom_gates {
                    Some(_) => panic!("multiple custom gates pragmas"),
                    None => custom_gates = Some(true),
                },
                // should be caught in parser
                _ => unreachable!(),
            }
        }

        let custom_gates_declared = definitions.iter().any(|definition| {
            matches!(
                definition,
                Definition::Template {
                    is_custom_gate: true,
                    ..
                }
            )
        });

        AST {
            meta,
            compiler_version,
            custom_gates: custom_gates.unwrap_or(false),
            custom_gates_declared,
            includes,
            definitions,
            main_component,
        }
    }

    pub fn get_includes(&self) -> &Vec<String> {
        &self.includes
    }

    pub fn get_version(&self) -> &Option<Version> {
        &self.compiler_version
    }

    pub fn get_definitions(&self) -> &Vec<Definition> {
        &self.definitions
    }

    pub fn decompose(
        self,
    ) -> (
        Meta,
        Option<Version>,
        Vec<String>,
        Vec<Definition>,
        Option<MainComponent>,
    ) {
        (
            self.meta,
            self.compiler_version,
            self.includes,
            self.definitions,
            self.main_component,
        )
    }
}
