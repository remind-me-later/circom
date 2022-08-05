use super::analysis_utilities::*;
use super::component_preprocess;
use super::sugar_cleaner;
use super::very_concrete_program::*;
use circom_ast::*;
use program_structure::program_archive::ProgramArchive;

pub fn run_preprocessing(vcp: &mut VCP, program_archive: ProgramArchive) {
    let mut state = build_function_knowledge(program_archive);
    produce_vcf(vcp, &mut state);
    link_circuit(vcp, &mut state);
    vcp.quick_knowledge = state.quick_knowledge;
    vcp.functions = state.vcf_collector;
    component_preprocess::rm_component_ci(vcp);
    sugar_cleaner::clean_sugar(vcp);
}

fn produce_vcf(vcp: &VCP, state: &mut State) {
    for n in &vcp.templates {
        let code = &n.code;
        let constants = &n.header;
        let params = vec![];
        state.external_signals = build_component_info(&n.triggers);
        let mut env = build_environment(constants, &params);
        produce_vcf_stmt(code, state, &mut env);
    }
    let mut index = 0;
    while index < state.vcf_collector.len() {
        state.external_signals = build_component_info(&vec![]);
        let mut env = build_environment(&vec![], &state.vcf_collector[index].params_types);
        let body = state.vcf_collector[index].body.clone();
        produce_vcf_stmt(&body, state, &mut env);
        index += 1;
    }
}

fn link_circuit(vcp: &mut VCP, state: &mut State) {
    for node in &mut vcp.templates {
        let mut env = build_environment(&node.header, &vec![]);
        state.external_signals = build_component_info(&node.triggers);
        link_stmt(&mut node.code, state, &mut env);
    }
    let mut linked_vcf_collector = state.vcf_collector.clone();
    for vcf in &mut linked_vcf_collector {
        let mut env = build_environment(&Vec::new(), &vcf.params_types);
        link_stmt(&mut vcf.body, state, &mut env);
    }
    state.vcf_collector = linked_vcf_collector;
}

// Function knowledge handling
fn add_instance(name: &str, args: Vec<Param>, state: &mut State) {
    use super::type_inference::infer_function_result;
    let already_exits = look_for_existing_instance(name, &args, state);
    if already_exits.is_none() {
        let inferred = infer_function_result(name, args.clone(), state);
        let id = state.vcf_collector.len();
        let body = state.generic_functions.get(name).unwrap().body.clone();
        let new_vcf = VCF {
            name: name.to_string(),
            header: format!("{}_{}", name, state.vcf_collector.len()),
            params_types: args.to_vec(),
            return_type: inferred,
            body,
        };
        state.quick_knowledge.insert(new_vcf.header.clone(), new_vcf.return_type.clone());
        state.vcf_collector.push(new_vcf);
        state.generic_functions.get_mut(name).unwrap().concrete_instances.push(id);
    }
}

fn look_for_existing_instance(
    name: &str,
    args: &Vec<Param>,
    state: &State,
) -> Option<(usize, VCT)> {
    let vcf = state.generic_functions.get(name).unwrap();
    for i in &vcf.concrete_instances {
        let return_type = &state.vcf_collector[*i].return_type;
        let params_types = &state.vcf_collector[*i].params_types;
        if params_types.eq(args) {
            return Option::Some((*i, return_type.clone()));
        }
    }
    Option::None
}

// Algorithm for producing the code's vcf
fn produce_vcf_stmt(stmt: &Statement, state: &mut State, environment: &mut E) {
    use Statement::*;
    match stmt {
        Return { value, .. } => produce_vcf_expr(value, state, environment),
        Assert { arg, .. } => produce_vcf_expr(arg, state, environment),
        LogCall { arg, .. } => produce_vcf_expr(arg, state, environment),
        ConstraintEquality { lhe, rhe, .. } => {
            produce_vcf_expr(lhe, state, environment);
            produce_vcf_expr(rhe, state, environment);
        }
        Substitution { access, rhe, .. } => {
            produce_vcf_expr(rhe, state, environment);
            for a in access {
                if let Access::ArrayAccess(index) = a {
                    produce_vcf_expr(index, state, environment);
                }
            }
        }
        Declaration { name, dimensions, meta, .. } => {
            for d in dimensions {
                produce_vcf_expr(d, state, environment);
            }
            let with_type = meta.get_memory_knowledge().get_concrete_dimensions().to_vec();
            environment.add_variable(name, with_type);
        }
        Block { stmts, .. } => {
            environment.add_variable_block();
            for s in stmts {
                produce_vcf_stmt(s, state, environment);
            }
            environment.remove_variable_block();
        }
        InitializationBlock { initializations, .. } => {
            for s in initializations {
                produce_vcf_stmt(s, state, environment);
            }
        }
        While { cond, stmt, .. } => {
            produce_vcf_expr(cond, state, environment);
            produce_vcf_stmt(stmt, state, environment);
        }
        IfThenElse { cond, if_case, else_case, .. } => {
            produce_vcf_expr(cond, state, environment);
            produce_vcf_stmt(if_case, state, environment);
            if let Option::Some(s) = else_case {
                produce_vcf_stmt(s, state, environment);
            }
        }
    }
}

fn produce_vcf_expr(expr: &Expression, state: &mut State, environment: &E) {
    use Expression::*;
    match expr {
        Number { .. } => (),
        InfixOp { lhe, rhe, .. } => {
            produce_vcf_expr(lhe, state, environment);
            produce_vcf_expr(rhe, state, environment);
        }
        PrefixOp { rhe, .. } => {
            produce_vcf_expr(rhe, state, environment);
        }
        TernaryOp { if_true, if_false, .. } => {
            produce_vcf_expr(if_true, state, environment);
            produce_vcf_expr(if_false, state, environment);
        }
        Variable { access, .. } => {
            for a in access {
                if let Access::ArrayAccess(index) = a {
                    produce_vcf_expr(index, state, environment);
                }
            }
        }
        Call { id, args, .. } => {
            for arg in args {
                produce_vcf_expr(arg, state, environment);
            }
            if state.generic_functions.contains_key(id) {
                let params = map_to_params(id, args, state, environment);
                add_instance(id, params, state);
            }
        }
        ArrayInLine { values, .. } => {
            for v in values {
                produce_vcf_expr(v, state, environment);
            }
        }
    }
}

/*
    Linking algorithm:
    The linking algorithm not only changes
    functions call to vcf calls. Also, every expression
    will be linked with its vct (very concrete type).
*/
fn link_stmt(stmt: &mut Statement, state: &State, env: &mut E) {
    use Statement::*;
    match stmt {
        Block { stmts, .. } => {
            env.add_variable_block();
            stmts.iter_mut().for_each(|s| link_stmt(s, state, env));
            env.remove_variable_block();
        }
        InitializationBlock { initializations, .. } => {
            initializations.iter_mut().for_each(|i| link_stmt(i, state, env));
        }
        IfThenElse { cond, if_case, else_case, .. } => {
            link_expression(cond, state, env);
            link_stmt(if_case, state, env);
            if let Option::Some(s) = else_case {
                link_stmt(s, state, env);
            }
        }
        While { cond, stmt, .. } => {
            link_expression(cond, state, env);
            link_stmt(stmt, state, env);
        }
        LogCall { arg, .. } => link_expression(arg, state, env),
        Assert { arg, .. } => link_expression(arg, state, env),
        Return { value, .. } => link_expression(value, state, env),
        ConstraintEquality { lhe, rhe, .. } => {
            link_expression(lhe, state, env);
            link_expression(rhe, state, env);
        }
        Declaration { name, dimensions, meta, .. } => {
            dimensions.iter_mut().for_each(|d| link_expression(d, state, env));
            let has_type = meta.get_memory_knowledge().get_concrete_dimensions().to_vec();
            env.add_variable(name, has_type);
        }
        Substitution { access, rhe, .. } => {
            link_expression(rhe, state, env);
            for acc in access {
                if let Access::ArrayAccess(e) = acc {
                    link_expression(e, state, env);
                }
            }
        }
    }
}

fn link_expression(expr: &mut Expression, state: &State, env: &E) {
    use Expression::*;

    match expr {
        Number { .. } => (),
        Call { id, args, .. } => {
            args.iter_mut().for_each(|a| link_expression(a, state, env));

            if state.generic_functions.contains_key(id) {
                let params = map_to_params(id, args, state, env);
                let (index, _) = look_for_existing_instance(id, &params, state).unwrap();
                *id = state.vcf_collector[index].header.clone();
            }
        }
        ArrayInLine { values, .. } => {
            values.iter_mut().for_each(|v| link_expression(v, state, env))
        }
        Variable { access, .. } => {
            for acc in access {
                if let Access::ArrayAccess(e) = acc {
                    link_expression(e, state, env);
                }
            }
        }
        TernaryOp { if_true, if_false, .. } => {
            link_expression(if_true, state, env);
            link_expression(if_false, state, env);
        }
        InfixOp { lhe, rhe, .. } => {
            link_expression(lhe, state, env);
            link_expression(rhe, state, env);
        }
        PrefixOp { rhe, .. } => link_expression(rhe, state, env),
    }

    let has_type = cast_type_expression(expr, state, env);
    expr.get_mut_meta().get_mut_memory_knowledge().set_concrete_dimensions(has_type);
}

// When the vcf of a branch in the AST had been produced, this algorithm
// can return the type of a expression in that branch
fn cast_type_expression(expr: &Expression, state: &State, environment: &E) -> VCT {
    use Expression::*;
    match expr {
        Variable { name, access, .. } => {
            let mut xtype = environment.get_variable(name).unwrap().clone();
            xtype.reverse();
            for acc in access {
                match acc {
                    Access::ArrayAccess(_) => {
                        xtype.pop();
                    }
                    Access::ComponentAccess(signal) => {
                        xtype =
                            state.external_signals.get(name).unwrap().get(signal).unwrap().clone();
                        xtype.reverse();
                    }
                }
            }
            xtype.reverse();
            xtype
        }
        ArrayInLine { values, .. } => {
            let mut result = vec![values.len()];
            let mut inner_type = cast_type_expression(&values[0], state, environment);
            result.append(&mut inner_type);
            result
        }
        Call { id, args, .. } => {
            if let Option::Some(returns) = state.quick_knowledge.get(id) {
                returns.clone()
            } else if !state.generic_functions.contains_key(id) {
                vec![]
            } else {
                let params = map_to_params(id, args, state, environment);
                look_for_existing_instance(id, &params, state).unwrap().1
            }
        }
        TernaryOp { if_true, .. } => cast_type_expression(if_true, state, environment),
        _ => VCT::with_capacity(0),
    }
}

fn map_to_params(function_id: &str, args: &[Expression], state: &State, env: &E) -> Vec<Param> {
    let mut params = vec![];
    let names = &state.generic_functions.get(function_id).unwrap().params_names;
    let mut index = 0;
    while index < args.len() {
        let param = Param {
            name: names[index].clone(),
            length: cast_type_expression(&args[index], state, env),
        };
        params.push(param);
        index += 1;
    }
    params
}
