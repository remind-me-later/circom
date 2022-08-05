use circom_ast::VariableType;
use circom_ast::{FillMeta, SignalElementType, Statement};
use circom_error::file_definition::FileID;
use circom_error::file_definition::LocationInFile;
use std::collections::hash_map::HashMap;

pub type TemplateInfo = HashMap<String, TemplateData>;
type SignalInfo = HashMap<String, (usize, SignalElementType)>;

#[derive(Clone)]
pub struct TemplateData {
    file_id: FileID,
    name: String,
    body: Statement,
    num_of_params: usize,
    name_of_params: Vec<String>,
    param_location: LocationInFile,
    input_signals: SignalInfo,
    output_signals: SignalInfo,
    is_parallel: bool,
    is_custom_gate: bool,
}

impl TemplateData {
    pub fn new(
        name: String,
        file_id: FileID,
        mut body: Statement,
        num_of_params: usize,
        name_of_params: Vec<String>,
        param_location: LocationInFile,
        elem_id: &mut usize,
        is_parallel: bool,
        is_custom_gate: bool,
    ) -> TemplateData {
        body.fill(file_id, elem_id);
        let mut input_signals = SignalInfo::new();
        let mut output_signals = SignalInfo::new();
        fill_inputs_and_outputs(&body, &mut input_signals, &mut output_signals);
        TemplateData {
            name,
            file_id,
            body,
            num_of_params,
            name_of_params,
            param_location,
            input_signals,
            output_signals,
            is_parallel,
            is_custom_gate,
        }
    }
    pub fn get_file_id(&self) -> FileID {
        self.file_id
    }
    pub fn get_body(&self) -> &Statement {
        &self.body
    }
    pub fn get_body_as_vec(&self) -> &Vec<Statement> {
        match &self.body {
            Statement::Block { stmts, .. } => stmts,
            _ => panic!("Function body should be a block"),
        }
    }
    pub fn get_mut_body(&mut self) -> &mut Statement {
        &mut self.body
    }
    pub fn get_mut_body_as_vec(&mut self) -> &mut Vec<Statement> {
        match &mut self.body {
            Statement::Block { stmts, .. } => stmts,
            _ => panic!("Function body should be a block"),
        }
    }
    pub fn get_num_of_params(&self) -> usize {
        self.num_of_params
    }
    pub fn get_param_location(&self) -> LocationInFile {
        self.param_location.clone()
    }
    pub fn get_name_of_params(&self) -> &Vec<String> {
        &self.name_of_params
    }
    pub fn get_input_info(&self, name: &str) -> Option<&(usize, SignalElementType)> {
        self.input_signals.get(name)
    }
    pub fn get_output_info(&self, name: &str) -> Option<&(usize, SignalElementType)> {
        self.output_signals.get(name)
    }
    pub fn get_inputs(&self) -> &SignalInfo {
        &self.input_signals
    }
    pub fn get_outputs(&self) -> &SignalInfo {
        &self.output_signals
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn is_parallel(&self) -> bool {
        self.is_parallel
    }
    pub fn is_custom_gate(&self) -> bool {
        self.is_custom_gate
    }
}

fn fill_inputs_and_outputs(
    template_statement: &Statement,
    input_signals: &mut SignalInfo,
    output_signals: &mut SignalInfo,
) {
    match template_statement {
        Statement::IfThenElse { if_case, else_case, .. } => {
            fill_inputs_and_outputs(if_case, input_signals, output_signals);
            if let Some(else_value) = else_case {
                fill_inputs_and_outputs(else_value, input_signals, output_signals);
            }
        }
        Statement::Block { stmts, .. } => {
            for stmt in stmts.iter() {
                fill_inputs_and_outputs(stmt, input_signals, output_signals);
            }
        }
        Statement::While { stmt, .. } => {
            fill_inputs_and_outputs(stmt, input_signals, output_signals);
        }
        Statement::InitializationBlock { initializations, .. } => {
            for initialization in initializations.iter() {
                fill_inputs_and_outputs(initialization, input_signals, output_signals);
            }
        }
        Statement::Declaration {
            xtype: VariableType::Signal(stype, tag),
            name,
            dimensions,
            ..
        } => {
            let signal_name = name.clone();
            let dim = dimensions.len();
            match stype {
                circom_ast::SignalType::Input => {
                    input_signals.insert(signal_name, (dim, *tag));
                }
                circom_ast::SignalType::Output => {
                    output_signals.insert(signal_name, (dim, *tag));
                }
                _ => {} //no need to deal with intermediate signals
            }
        }
        _ => {}
    }
}
