use circom_ast::{Statement, VariableType};
use circom_error::error_code::ReportCode;
use circom_error::error_definition::{Report, ReportCollection};
use circom_error::file_definition::{self, FileID};
use program_structure::template_data::TemplateData;

pub fn check_signal_correctness(template_data: &TemplateData) -> Result<(), ReportCollection> {
    let template_id = template_data.get_file_id();
    let template_body = template_data.get_body_as_vec();
    let mut reports = ReportCollection::new();
    for stmt in template_body.iter() {
        treat_statement(stmt, true, template_id, &mut reports);
    }
    if reports.is_empty() {
        Ok(())
    } else {
        Err(reports)
    }
}

fn treat_statement(
    stmt: &Statement,
    signal_declaration_allowed: bool,
    template_id: FileID,
    reports: &mut ReportCollection,
) {
    use Statement::*;
    match stmt {
        IfThenElse {
            if_case, else_case, ..
        } => {
            treat_statement(if_case, false, template_id, reports);
            if let Some(else_block) = else_case {
                treat_statement(else_block, false, template_id, reports);
            }
        }
        While { stmt, .. } => {
            treat_statement(stmt, false, template_id, reports);
        }
        Block { stmts, .. } => {
            for stmt in stmts.iter() {
                treat_statement(stmt, false, template_id, reports);
            }
        }
        InitializationBlock { meta, xtype, .. } => match xtype {
            VariableType::Signal(_, _) | VariableType::Component => {
                if !signal_declaration_allowed {
                    let mut report = Report::error(
                        "Signal or component declaration outside initial scope".to_string(),
                        ReportCode::SignalOutsideOriginalScope,
                    );
                    let location =
                        file_definition::generate_file_location(meta.get_start(), meta.get_end());
                    report.add_primary(
                        location,
                        template_id,
                        "Is outside the initial scope".to_string(),
                    );
                    reports.push(report);
                }
            }
            _ => {}
        },
        _ => {}
    };
}
