use program_structure::ast::Statement;
use circom_error::error_code::ReportCode;
use circom_error::error_definition::{Report, ReportCollection};
use program_structure::function_data::FunctionData;

pub fn all_paths_with_return_check(function_data: &FunctionData, reports: &mut ReportCollection) {
    let function_body = function_data.get_body();
    let function_name = function_data.get_name();

    if !analyse_statement(function_body) {
        reports.push(Report::error(
            format!("In {} there are paths without return", function_name),
            ReportCode::FunctionReturnError,
        ));
    }
}

fn analyse_statement(stmt: &Statement) -> bool {
    match stmt {
        Statement::Return { .. } => true,
        Statement::While { .. } => false,
        Statement::IfThenElse { if_case, else_case, .. } => {
            let else_returns = match else_case {
                Option::Some(s) => analyse_statement(s),
                _ => false,
            };
            else_returns && analyse_statement(if_case)
        }
        Statement::Block { stmts, .. } => analyse_block(stmts),
        _ => false,
    }
}

fn analyse_block(block: &[Statement]) -> bool {
    let mut has_return_path = false;
    for stmt in block.iter() {
        has_return_path = has_return_path || analyse_statement(stmt);
    }
    has_return_path
}
