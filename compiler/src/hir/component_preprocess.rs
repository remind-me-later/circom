use crate::hir::very_concrete_program::VCP;
use circom_ast::*;

pub fn rm_component_ci(vcp: &mut VCP) {
    vcp.templates.iter_mut().for_each(|t| rm_statement(&mut t.code));
}

fn should_be_removed(stmt: &Statement) -> bool {
    use Statement::{InitializationBlock, Substitution};
    use VariableType::*;
    if let InitializationBlock { xtype, .. } = stmt {
        Component == *xtype
    } else if let Substitution { meta, .. } = stmt {
        meta.get_type_knowledge().is_component()
    } else {
        false
    }
}

fn rm_statement(stmt: &mut Statement) {
    use Statement::*;
    match stmt {
        While { stmt, .. } => {
            rm_statement(stmt);
        }
        IfThenElse { if_case, else_case, .. } => {
            rm_statement(if_case);
            if let Some(s) = else_case {
                rm_statement(s);
            }
        }
        Block { stmts, .. } => {
            let filter = std::mem::take(stmts);
            for mut s in filter {
                rm_statement(&mut s);
                if !should_be_removed(&s) {
                    stmts.push(s);
                }
            }
        }
        InitializationBlock { initializations, xtype, .. } => {
            if let VariableType::Signal(..) = xtype {
                let work = std::mem::take(initializations);
                for i in work {
                    if matches!(i, Substitution { .. }) {
                        initializations.push(i);
                    }
                }
            }
        }
        Substitution { .. } => {
            if should_be_removed(stmt) {
                if let Substitution { meta, .. } = stmt {
                    *stmt = Block { meta: meta.clone(), stmts: Vec::new() };
                }
            }
        }
        _ => (),
    }
}
