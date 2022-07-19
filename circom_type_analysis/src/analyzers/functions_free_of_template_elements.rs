use program_structure::ast::*;
use circom_error::error_code::ReportCode;
use circom_error::error_definition::{Report, ReportCollection};
use circom_error::file_definition;
use program_structure::function_data::{FunctionData, FunctionInfo};

pub fn free_of_template_elements(
    function_data: &FunctionData,
    functions: &FunctionInfo,
    reports: &mut ReportCollection,
) {
    let body = function_data.get_body();
    analyse_statement(body, functions, reports);
}

fn analyse_statement(stmt: &Statement, functions: &FunctionInfo, reports: &mut ReportCollection) {
    use Statement::*;
    let file_id = stmt.get_meta().get_file_id();
    match stmt {
        IfThenElse { cond, if_case, else_case, .. } => {
            analyse_expression(cond, functions, reports);
            analyse_statement(if_case, functions, reports);
            if let Option::Some(else_block) = else_case {
                analyse_statement(else_block, functions, reports);
            }
        }
        While { cond, stmt, .. } => {
            analyse_expression(cond, functions, reports);
            analyse_statement(stmt, functions, reports);
        }
        Block { stmts, .. } => {
            for stmt in stmts.iter() {
                analyse_statement(stmt, functions, reports);
            }
        }
        InitializationBlock { meta, xtype, initializations } => {
            if let VariableType::Signal(..) = xtype {
                let mut report = Report::error(
                    "Template elements declared inside the function".to_string(),
                    ReportCode::UndefinedFunction,
                );
                let location =
                    file_definition::generate_file_location(meta.get_start(), meta.get_end());
                report.add_primary(location, file_id, "Declaring template element".to_string());
                reports.push(report);
                return;
            }
            for initialization in initializations.iter() {
                analyse_statement(initialization, functions, reports);
            }
        }
        Declaration { meta, xtype, dimensions, .. } => {
            if let VariableType::Var = xtype {
                for dimension in dimensions.iter() {
                    analyse_expression(dimension, functions, reports);
                }
            } else {
                let mut report = Report::error(
                    "Template elements declared inside the function".to_string(),
                    ReportCode::UndefinedFunction,
                );
                let location =
                    file_definition::generate_file_location(meta.get_start(), meta.get_end());
                report.add_primary(location, file_id, "Declaring template element".to_string());
                reports.push(report);
            }
        }
        Substitution { meta, op, access, rhe, .. } => {
            if op.is_signal_operator() {
                let mut report = Report::error(
                    "Function uses template operators".to_string(),
                    ReportCode::UndefinedFunction,
                );
                let location =
                    file_definition::generate_file_location(meta.get_start(), meta.get_end());
                report.add_primary(location, file_id, "Template operator found".to_string());
                reports.push(report);
            }
            analyse_access(access, meta, functions, reports);
            analyse_expression(rhe, functions, reports);
        }
        ConstraintEquality { meta, lhe, rhe, .. } => {
            let mut report = Report::error(
                "Function uses template operators".to_string(),
                ReportCode::UndefinedFunction,
            );
            let location =
                file_definition::generate_file_location(meta.get_start(), meta.get_end());
            report.add_primary(location, file_id, "Template operator found".to_string());
            reports.push(report);
            analyse_expression(lhe, functions, reports);
            analyse_expression(rhe, functions, reports);
        }
        LogCall { arg, .. } | Assert { arg, .. } => {
            analyse_expression(arg, functions, reports);
        }
        Return { value, .. } => {
            analyse_expression(value, functions, reports);
        }
    }
}

fn analyse_access(
    access: &[Access],
    meta: &Meta,
    functions: &FunctionInfo,
    reports: &mut ReportCollection,
) {
    let file_id = meta.get_file_id();
    for acc in access.iter() {
        if let Access::ArrayAccess(index) = acc {
            analyse_expression(index, functions, reports);
        } else {
            let mut report = Report::error(
                "Function uses component operators".to_string(),
                ReportCode::UndefinedFunction,
            );
            let location =
                file_definition::generate_file_location(meta.get_start(), meta.get_end());
            report.add_primary(location, file_id, "Template operator found".to_string());
            reports.push(report);
        }
    }
}

fn analyse_expression(expr: &Expression, functions: &FunctionInfo, reports: &mut ReportCollection) {
    use Expression::*;
    let file_id = expr.get_meta().get_file_id();
    match expr {
        InfixOp { lhe, rhe, .. } => {
            analyse_expression(lhe, functions, reports);
            analyse_expression(rhe, functions, reports);
        }
        PrefixOp { rhe, .. } => {
            analyse_expression(rhe, functions, reports);
        }
        InlineSwitchOp { cond, if_true, if_false, .. } => {
            analyse_expression(cond, functions, reports);
            analyse_expression(if_true, functions, reports);
            analyse_expression(if_false, functions, reports);
        }
        Variable { meta, access, .. } => analyse_access(access, meta, functions, reports),
        Number(..) => {}
        Call { meta, id, args, .. } => {
            if !functions.contains_key(id) {
                let mut report = Report::error(
                    "Unknown call in function".to_string(),
                    ReportCode::UndefinedFunction,
                );
                let location =
                    file_definition::generate_file_location(meta.get_start(), meta.get_end());
                report.add_primary(location, file_id, "Is not a function call".to_string());
                reports.push(report);
            }
            for arg in args.iter() {
                analyse_expression(arg, functions, reports);
            }
        }
        ArrayInLine { values, .. } => {
            for value in values.iter() {
                analyse_expression(value, functions, reports);
            }
        }
    }
}
