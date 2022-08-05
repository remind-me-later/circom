mod ast_impl;
pub mod ast_shortcuts;
mod expression_impl;
mod statement_impl;
pub mod knowledge;
mod expression;
mod statement;

use circom_error::file_definition::FileLocation;

pub use knowledge::{TypeKnowledge, MemoryKnowledge, TypeReduction};
pub use expression::{Expression, ExpressionPrefixOpcode, ExpressionInfixOpcode};
pub use statement::*;

pub trait FillMeta {
    fn fill(&mut self, file_id: usize, elem_id: &mut usize);
}

pub type MainComponent = (Vec<String>, Expression);
pub fn build_main_component(public: Vec<String>, call: Expression) -> MainComponent {
    (public, call)
}

pub type Version = (usize, usize, usize);

#[derive(Clone)]
pub struct Meta {
    pub elem_id: usize,
    pub start: usize,
    pub end: usize,
    pub location: FileLocation,
    pub file_id: Option<usize>,
    pub component_inference: Option<String>,
    type_knowledge: TypeKnowledge,
    memory_knowledge: MemoryKnowledge,
}

impl Meta {
    pub fn new(start: usize, end: usize) -> Meta {
        Meta {
            end,
            start,
            elem_id: 0,
            location: start..end,
            file_id: Option::None,
            component_inference: None,
            type_knowledge: TypeKnowledge::default(),
            memory_knowledge: MemoryKnowledge::default(),
        }
    }
    pub fn change_location(&mut self, location: FileLocation, file_id: Option<usize>) {
        self.location = location;
        self.file_id = file_id;
    }
    pub fn get_start(&self) -> usize {
        self.location.start
    }
    pub fn get_end(&self) -> usize {
        self.location.end
    }
    pub fn get_file_id(&self) -> usize {
        if let Option::Some(id) = self.file_id {
            id
        } else {
            panic!("Empty file id accessed")
        }
    }
    pub fn get_memory_knowledge(&self) -> &MemoryKnowledge {
        &self.memory_knowledge
    }
    pub fn get_type_knowledge(&self) -> &TypeKnowledge {
        &self.type_knowledge
    }
    pub fn get_mut_memory_knowledge(&mut self) -> &mut MemoryKnowledge {
        &mut self.memory_knowledge
    }
    pub fn get_mut_type_knowledge(&mut self) -> &mut TypeKnowledge {
        &mut self.type_knowledge
    }
    pub fn file_location(&self) -> FileLocation {
        self.location.clone()
    }
    pub fn set_file_id(&mut self, file_id: usize) {
        self.file_id = Option::Some(file_id);
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
        compiler_version: Option<Version>,
        custom_gates: bool,
        includes: Vec<String>,
        definitions: Vec<Definition>,
        main_component: Option<MainComponent>,
    ) -> AST {
        let custom_gates_declared = definitions.iter().any(|definition| {
            matches!(definition, Definition::Template { is_custom_gate: true, .. })
        });
        AST {
            meta,
            compiler_version,
            custom_gates,
            custom_gates_declared,
            includes,
            definitions,
            main_component,
        }
    }
}

#[derive(Clone)]
pub enum Definition {
    Template {
        meta: Meta,
        name: String,
        args: Vec<String>,
        arg_location: FileLocation,
        body: Statement,
        parallel: bool,
        is_custom_gate: bool,
    },
    Function {
        meta: Meta,
        name: String,
        args: Vec<String>,
        arg_location: FileLocation,
        body: Statement,
    },
}

impl Definition {
    pub fn build_template(
        meta: Meta,
        name: String,
        args: Vec<String>,
        arg_location: FileLocation,
        body: Statement,
        parallel: bool,
        is_custom_gate: bool,
    ) -> Definition {
        Definition::Template { meta, name, args, arg_location, body, parallel, is_custom_gate }
    }

    pub fn build_function(
        meta: Meta,
        name: String,
        args: Vec<String>,
        arg_location: FileLocation,
        body: Statement,
    ) -> Definition {
        Definition::Function { meta, name, args, arg_location, body }
    }
}
