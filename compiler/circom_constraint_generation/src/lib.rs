#![allow(clippy::result_unit_err)]

mod compute_constants;
mod environment_utils;
mod execute;
mod execution_data;

use ansi_term::Colour;
use circom_algebra::algebra::{ArithmeticError, ArithmeticExpression};
use circom_compiler::hir::very_concrete_program::VCP;
use circom_constraint_list::ConstraintList;
use circom_constraint_writers::ConstraintExporter;
use circom_dag::DAG;
use circom_error::error_code::ReportCode;
use circom_error::error_definition::{Report, ReportCollection};
use circom_error::file_definition::FileID;
use circom_program_structure::program_archive::ProgramArchive;
use execution_data::executed_program::ExportResult;
use execution_data::ExecutedProgram;
use std::rc::Rc;

pub struct BuildConfig {
    pub no_rounds: usize,
    pub flag_json_sub: bool,
    pub flag_s: bool,
    pub flag_f: bool,
    pub flag_p: bool,
    pub flag_verbose: bool,
    pub inspect_constraints: bool,
    pub prime: String,
}

pub type ConstraintWriter = Box<dyn ConstraintExporter>;
type BuildResponse = Result<(ConstraintWriter, VCP), ()>;
pub fn build_circuit(program: ProgramArchive, config: BuildConfig) -> BuildResponse {
    let files = program.file_library.clone();
    let exe = instantiation(&program, config.flag_verbose, &config.prime).map_err(|r| {
        Report::print_reports(&r, &files);
    })?;
    let (mut dag, mut vcp, warnings) = export(exe, program, config.flag_verbose).map_err(|r| {
        Report::print_reports(&r, &files);
    })?;
    if config.inspect_constraints {
        Report::print_reports(&warnings, &files);
    }
    if config.flag_f {
        sync_dag_and_vcp(&mut vcp, &mut dag);
        Ok((Box::new(dag), vcp))
    } else {
        let list = simplification_process(&mut vcp, dag, &config);
        Ok((Box::new(list), vcp))
    }
}

type InstantiationResponse = Result<ExecutedProgram, ReportCollection>;
fn instantiation(
    program: &ProgramArchive,
    flag_verbose: bool,
    prime: &String,
) -> InstantiationResponse {
    let execution_result = execute::constraint_execution(program, flag_verbose, prime);
    match execution_result {
        Ok(program_exe) => {
            let no_nodes = program_exe.number_of_nodes();
            let success = Colour::Green.paint("template instances");
            let nodes_created = format!("{}: {}", success, no_nodes);
            println!("{}", &nodes_created);
            InstantiationResponse::Ok(program_exe)
        }
        Err(reports) => InstantiationResponse::Err(reports),
    }
}

fn export(exe: ExecutedProgram, program: ProgramArchive, flag_verbose: bool) -> ExportResult {
    exe.export(program, flag_verbose)
}

fn sync_dag_and_vcp(vcp: &mut VCP, dag: &mut DAG) {
    let witness = Rc::new(DAG::produce_witness(dag));
    VCP::add_witness_list(vcp, Rc::clone(&witness));
}

fn simplification_process(vcp: &mut VCP, dag: DAG, config: &BuildConfig) -> ConstraintList {
    use circom_dag::SimplificationFlags;
    let flags = SimplificationFlags {
        flag_s: config.flag_s,
        parallel_flag: config.flag_p,
        port_substitution: config.flag_json_sub,
        no_rounds: config.no_rounds,
        prime: config.prime.clone(),
    };
    let list = DAG::map_to_list(dag, flags);
    VCP::add_witness_list(vcp, Rc::new(list.get_witness_as_vec()));
    list
}
