use circom_ast::*;
use circom_error::error_code::ReportCode;
use circom_error::error_definition::{Report, ReportCollection};

pub fn custom_gate_analysis(
    custom_gate_name: &str,
    custom_gate_body: &Statement,
) -> Result<ReportCollection, ReportCollection> {
    fn custom_gate_analysis(
        custom_gate_name: &str,
        stmt: &Statement,
        errors: &mut ReportCollection,
        warnings: &mut ReportCollection,
    ) {
        use Statement::*;
        match stmt {
            IfThenElse {
                if_case, else_case, ..
            } => {
                custom_gate_analysis(custom_gate_name, if_case, warnings, errors);
                if let Some(else_case_s) = else_case {
                    custom_gate_analysis(custom_gate_name, else_case_s, errors, warnings);
                }
            }
            While { stmt, .. } => {
                custom_gate_analysis(custom_gate_name, stmt, errors, warnings);
            }
            InitializationBlock {
                initializations, ..
            } => {
                for stmt in initializations {
                    custom_gate_analysis(custom_gate_name, stmt, errors, warnings);
                }
            }
            Declaration {
                meta, xtype, name, ..
            } => {
                use VariableType::*;
                match xtype {
                    Signal(SignalType::Intermediate, _) => {
                        let mut warning = Report::warning(
                            String::from("Intermediate signal inside custom gate"),
                            ReportCode::CustomGateIntermediateSignalWarning,
                        );
                        warning.add_primary(
                            meta.location().clone(),
                            meta.unwrap_file_id(),
                            format!(
                                "Intermediate signal {} declared in custom gate {}",
                                name, custom_gate_name
                            ),
                        );
                        warnings.push(warning);
                    }
                    Component => {
                        let mut error = Report::error(
                            String::from("Component inside custom gate"),
                            ReportCode::CustomGateSubComponent,
                        );
                        error.add_primary(
                            meta.location().clone(),
                            meta.unwrap_file_id(),
                            format!(
                                "Component {} declared in custom gate {}",
                                name, custom_gate_name
                            ),
                        );
                        errors.push(error);
                    }
                    _ => {}
                }
            }
            Substitution { meta, op, .. } => {
                use AssignOp::*;
                if let AssignConstraintSignal = op {
                    let mut error = Report::error(
                        String::from("Added constraint inside custom gate"),
                        ReportCode::CustomGateConstraint,
                    );
                    error.add_primary(
                        meta.location().clone(),
                        meta.unwrap_file_id(),
                        String::from("Added constraint"),
                    );
                    errors.push(error);
                }
            }
            ConstraintEquality { meta, .. } => {
                let mut error = Report::error(
                    String::from("Added constraint inside custom gate"),
                    ReportCode::CustomGateConstraint,
                );
                error.add_primary(
                    meta.location().clone(),
                    meta.unwrap_file_id(),
                    String::from("Added constraint"),
                );
                errors.push(error);
            }
            Block { stmts, .. } => {
                for stmt in stmts {
                    custom_gate_analysis(custom_gate_name, stmt, errors, warnings);
                }
            }
            _ => {}
        };
    }

    let mut warnings = vec![];
    let mut errors = vec![];

    custom_gate_analysis(
        custom_gate_name,
        custom_gate_body,
        &mut warnings,
        &mut errors,
    );

    if errors.is_empty() {
        Ok(warnings)
    } else {
        Err(errors)
    }
}
