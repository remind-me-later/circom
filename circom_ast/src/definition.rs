use circom_error::file_definition::FileLocation;

use crate::{Statement, Meta};

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
