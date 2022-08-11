use super::analysis_utilities::*;
use super::very_concrete_program::*;
use circom_ast::*;
use std::collections::HashSet;

struct SearchInfo {
    environment: E,
    open_calls: HashSet<String>,
}

pub fn infer_function_result(id: &str, params: Vec<Param>, state: &State) -> VCT {
    let body = &state.generic_functions.get(id).unwrap().body;
    let mut context = SearchInfo {
        environment: E::new(),
        open_calls: HashSet::new(),
    };
    context.open_calls.insert(id.to_string());

    params
        .into_iter()
        .for_each(|p| context.environment.add_variable(&p.name, p.length));

    infer_type_stmt(body, state, &mut context).unwrap()
}

fn infer_type_stmt(stmt: &Statement, state: &State, context: &mut SearchInfo) -> Option<VCT> {
    use Statement::*;
    match stmt {
        Return { value, .. } => infer_type_expresion(value, state, context),
        Block { stmts, .. } => {
            let mut returns = None;
            let mut index = 0;
            context.environment.add_variable_block();
            while index < stmts.len() && returns.is_none() {
                returns = infer_type_stmt(&stmts[index], state, context);
                index += 1;
            }
            context.environment.remove_variable_block();
            returns
        }
        InitializationBlock {
            initializations, ..
        } => {
            for s in initializations {
                if let Declaration { meta, name, .. } = s {
                    let has_type = meta
                        .get_memory_knowledge()
                        .get_concrete_dimensions()
                        .to_vec();
                    context.environment.add_variable(name, has_type);
                };
            }

            None
        }
        IfThenElse {
            if_case, else_case, ..
        } => {
            let mut returns = infer_type_stmt(if_case, state, context);
            if let Some(s) = else_case {
                if returns.is_none() {
                    returns = infer_type_stmt(s, state, context);
                }
            }
            returns
        }
        While { stmt, .. } => infer_type_stmt(stmt, state, context),
        Declaration { meta, name, .. } => {
            let has_type = meta
                .get_memory_knowledge()
                .get_concrete_dimensions()
                .to_vec();
            context.environment.add_variable(name, has_type);
            None
        }
        _ => None,
    }
}

fn infer_type_expresion(expr: &Expression, state: &State, context: &mut SearchInfo) -> Option<VCT> {
    use Expression::*;
    match expr {
        Call { id, args, .. } => {
            if context.open_calls.contains(id) {
                None
            } else {
                context.open_calls.insert(id.clone());
                context.environment.add_variable_block();
                let body = &state.generic_functions.get(id).unwrap().body;
                let names = &state.generic_functions.get(id).unwrap().params_names;
                let arg_types = infer_args(args, state, context);

                arg_types.as_ref()?;

                let arg_types = arg_types.unwrap();
                for (index, arg_type) in arg_types.into_iter().enumerate() {
                    context.environment.add_variable(&names[index], arg_type);
                }

                let inferred = infer_type_stmt(body, state, context);
                context.environment.remove_variable_block();
                context.open_calls.remove(id);
                inferred
            }
        }
        Variable { name, access, .. } => {
            let with_type = context.environment.get_variable(name).unwrap();
            Some(with_type[access.len()..].to_vec())
        }
        ArrayInLine { values, .. } => {
            let mut lengths = vec![values.len()];

            let with_type = infer_type_expresion(values.first().unwrap(), state, context);

            with_type.map(|ref mut l| {
                lengths.append(l);
                lengths
            })
        }
        TernaryOp {
            if_true, if_false, ..
        } => infer_type_expresion(if_true, state, context)
            .or_else(|| infer_type_expresion(if_false, state, context)),
        _ => Some(VCT::with_capacity(0)),
    }
}

fn infer_args(args: &[Expression], state: &State, context: &mut SearchInfo) -> Option<Vec<VCT>> {
    let mut arg_types = vec![];

    for arg in args {
        let arg_type = infer_type_expresion(arg, state, context);

        if let Some(t) = arg_type {
            arg_types.push(t);
        } else {
            return None;
        }
    }

    Some(arg_types)
}
