use super::very_concrete_program::*;
use circom_ast::*;

struct ExtendedSyntax {
    initializations: Vec<Statement>,
}

struct Context {}

struct State {
    fresh_id: usize,
}

impl State {
    pub fn produce_id(&mut self) -> String {
        let fresh = self.fresh_id;
        self.fresh_id += 1;
        format!("{}_auto", fresh)
    }
}

/*
    Tasks fulfilled by this algorithm:
        -Nested call removal
        -Inline if-then-else removal
        -Inline array removal
        -Initialization Block removal (no longer needed)
*/
pub fn clean_sugar(vcp: &mut VCP) {
    let mut state = State { fresh_id: 0 };

    for template in &mut vcp.templates {
        let context = Context {};
        let trash = extend_statement(&mut template.code, &mut state, &context);
        assert!(trash.is_empty());
    }

    for vcf in &mut vcp.functions {
        let context = Context {};
        let trash = extend_statement(&mut vcf.body, &mut state, &context);
        assert!(trash.is_empty());
    }
}

fn extend_statement(stmt: &mut Statement, state: &mut State, context: &Context) -> Vec<Statement> {
    use Statement::*;
    match stmt {
        Block { stmts, .. } => {
            let checkpoint_id = state.fresh_id;
            map_init_blocks(stmts);
            map_stmts_with_sugar(stmts, state, context);
            map_substitutions(stmts);
            state.fresh_id = map_returns(stmts, state.fresh_id);
            state.fresh_id = checkpoint_id;
            vec![]
        }
        While { cond, stmt, .. } => {
            let mut expands = extend_statement(stmt, state, context);
            let mut cond_extension = extend_expression(cond, state, context);
            expands.append(&mut cond_extension.initializations);
            let mut expr = vec![cond.clone()];
            sugar_filter(&mut expr, state, &mut expands);
            *cond = expr.pop().unwrap();
            expands
        }
        IfThenElse { cond, if_case, else_case, .. } => {
            let mut expands = vec![];
            expands.append(&mut extend_statement(if_case, state, context));
            if let Option::Some(s) = else_case {
                expands.append(&mut extend_statement(s, state, context));
            }
            let mut cond_extension = extend_expression(cond, state, context);
            expands.append(&mut cond_extension.initializations);
            let mut expr = vec![cond.clone()];
            sugar_filter(&mut expr, state, &mut expands);
            *cond = expr.pop().unwrap();
            expands
        }
        Substitution { rhe, .. } => extend_expression(rhe, state, context).initializations,
        ConstraintEquality { lhe, rhe, .. } => {
            let mut expands = extend_expression(lhe, state, context).initializations;
            expands.append(&mut extend_expression(rhe, state, context).initializations);
            expands
        }
        Declaration { .. } => vec![],
        Return { value, .. } => extend_expression(value, state, context).initializations,
        LogCall { arg, .. } => extend_expression(arg, state, context).initializations,
        Assert { arg, .. } => extend_expression(arg, state, context).initializations,
        _ => unreachable!(),
    }
}

fn extend_expression(
    expr: &mut Expression,
    state: &mut State,
    context: &Context,
) -> ExtendedSyntax {
    use Expression::*;
    match expr {
        Number { .. } => ExtendedSyntax { initializations: vec![] },
        Variable { access, .. } => {
            let mut initializations = vec![];
            for acc in access {
                if let Access::ArrayAccess(e) = acc {
                    let mut expand = extend_expression(e, state, context);
                    initializations.append(&mut expand.initializations);
                    let mut expr = vec![e.clone()];
                    sugar_filter(&mut expr, state, &mut initializations);
                    *e = expr.pop().unwrap();
                }
            }
            ExtendedSyntax { initializations }
        }
        ArrayInLine { values, .. } => {
            let mut initializations = vec![];
            for v in values.iter_mut() {
                let mut extended = extend_expression(v, state, context);
                initializations.append(&mut extended.initializations);
            }
            sugar_filter(values, state, &mut initializations);
            ExtendedSyntax { initializations }
        }
        Call { args, .. } => {
            let mut inits = vec![];
            for a in args.iter_mut() {
                let mut extended = extend_expression(a, state, context);
                inits.append(&mut extended.initializations);
            }
            sugar_filter(args, state, &mut inits);
            ExtendedSyntax { initializations: inits }
        }
        PrefixOp { rhe, .. } => {
            let mut extended = extend_expression(rhe, state, context);
            let mut expr = vec![*rhe.clone()];
            sugar_filter(&mut expr, state, &mut extended.initializations);
            *rhe = Box::new(expr.pop().unwrap());
            extended
        }
        InfixOp { lhe, rhe, .. } => {
            let mut lh_expand = extend_expression(lhe, state, context);
            let mut rh_expand = extend_expression(rhe, state, context);

            lh_expand.initializations.append(&mut rh_expand.initializations);
            let mut extended = lh_expand;
            let mut expr = vec![*lhe.clone(), *rhe.clone()];
            sugar_filter(&mut expr, state, &mut extended.initializations);
            *rhe = Box::new(expr.pop().unwrap());
            *lhe = Box::new(expr.pop().unwrap());
            extended
        }
        TernaryOp { if_true, if_false, .. } => {
            let mut true_expand = extend_expression(if_true, state, context);
            let mut false_expand = extend_expression(if_false, state, context);

            true_expand.initializations.append(&mut false_expand.initializations);
            let mut extended = true_expand;
            let mut expr = vec![*if_true.clone(), *if_false.clone()];
            sugar_filter(&mut expr, state, &mut extended.initializations);
            *if_false = Box::new(expr.pop().unwrap());
            *if_true = Box::new(expr.pop().unwrap());
            extended
        }
    }
}

// Utils
fn sugar_filter(elements: &mut Vec<Expression>, state: &mut State, inits: &mut Vec<Statement>) {
    let work = std::mem::replace(elements, Vec::with_capacity(elements.len()));
    for elem in work {
        let should_be_extracted = matches!(
            elem,
            Expression::Call { .. } | Expression::TernaryOp { .. } | Expression::ArrayInLine { .. }
        );
        if should_be_extracted {
            let id = state.produce_id();
            let expr = rmv_sugar(&id, elem, inits);
            elements.push(expr);
        } else {
            elements.push(elem);
        }
    }
}

fn rmv_sugar(fresh_variable: &str, expr: Expression, buffer: &mut Vec<Statement>) -> Expression {
    use Expression::Variable;
    use Statement::{Declaration, Substitution};
    let mut declaration_meta = expr.get_meta().clone();
    let mut variable_meta = expr.get_meta().clone();
    let mut initialization_meta = expr.get_meta().clone();
    declaration_meta.get_mut_type_knowledge().set_reduces_to(TypeReduction::Variable);
    variable_meta.get_mut_type_knowledge().set_reduces_to(TypeReduction::Variable);
    initialization_meta.get_mut_type_knowledge().set_reduces_to(TypeReduction::Variable);
    let declaration = Declaration {
        meta: declaration_meta,
        is_constant: true,
        xtype: VariableType::Var,
        name: fresh_variable.to_string(),
        dimensions: vec![],
    };
    let initialization = Substitution {
        meta: initialization_meta,
        var: fresh_variable.to_string(),
        access: vec![],
        op: AssignOp::AssignVar,
        rhe: expr,
    };
    let new_arg =
        Variable { meta: variable_meta, name: fresh_variable.to_string(), access: vec![] };
    buffer.push(declaration);
    buffer.push(initialization);
    new_arg
}

// mappings
fn map_stmts_with_sugar(stmts: &mut Vec<Statement>, state: &mut State, context: &Context) {
    let work = std::mem::take(stmts);
    for mut w in work {
        let mut inits = extend_statement(&mut w, state, context);
        stmts.append(&mut inits);
        stmts.push(w);
    }
}

fn map_init_blocks(stmts: &mut Vec<Statement>) {
    use Statement::InitializationBlock;
    let work = std::mem::take(stmts);
    for w in work {
        match w {
            InitializationBlock { mut initializations, .. } => stmts.append(&mut initializations),
            _ => stmts.push(w),
        }
    }
}

fn map_substitutions(stmts: &mut Vec<Statement>) {
    let work = std::mem::take(stmts);
    for w in work {
        match w {
            Statement::Substitution { .. } => into_single_substitution(w, stmts),
            _ => stmts.push(w),
        }
    }
}

fn map_returns(stmts: &mut Vec<Statement>, mut fresh_id: usize) -> usize {
    use Statement::Return;
    let work = std::mem::take(stmts);

    for w in work {
        let should_split = matches!(
            w,
            Return { value: Expression::ArrayInLine { .. }, .. }
                | Return { value: Expression::TernaryOp { .. }, .. }
                | Return { value: Expression::Call { .. }, .. }
        );

        if should_split {
            let split = split_return(w, fresh_id);
            stmts.push(split.declaration);
            into_single_substitution(split.substitution, stmts);
            stmts.push(split.final_return);
            fresh_id += 1;
        } else {
            stmts.push(w);
        }
    }

    fresh_id
}

struct ReturnSplit {
    declaration: Statement,
    substitution: Statement,
    final_return: Statement,
}

fn split_return(stmt: Statement, id: usize) -> ReturnSplit {
    use num_bigint_dig::BigInt;
    use Expression::{Number, Variable};
    use Statement::{Declaration, Return, Substitution};
    if let Return { value, meta } = stmt {
        let lengths = value.get_meta().get_memory_knowledge().get_concrete_dimensions().to_vec();
        let expr_lengths: Vec<Expression> = lengths
            .iter()
            .map(|v| Number { meta: meta.clone(), value: BigInt::from(*v) })
            .collect();
        let mut declaration_meta = meta.clone();
        let mut substitution_meta = meta.clone();
        let mut variable_meta = meta.clone();
        let return_meta = meta.clone();
        declaration_meta.get_mut_memory_knowledge().set_concrete_dimensions(lengths.clone());
        declaration_meta.get_mut_type_knowledge().set_reduces_to(TypeReduction::Variable);
        substitution_meta.get_mut_type_knowledge().set_reduces_to(TypeReduction::Variable);
        variable_meta.get_mut_memory_knowledge().set_concrete_dimensions(lengths);
        variable_meta.get_mut_type_knowledge().set_reduces_to(TypeReduction::Variable);
        let declaration = Declaration {
            meta: declaration_meta,
            xtype: VariableType::Var,
            name: id.to_string(),
            dimensions: expr_lengths,
            is_constant: false,
        };
        let substitution = Substitution {
            meta: substitution_meta,
            var: id.to_string(),
            access: vec![],
            op: AssignOp::AssignVar,
            rhe: value,
        };
        let returned_variable =
            Variable { meta: variable_meta, name: id.to_string(), access: vec![] };
        let final_return = Return { meta: return_meta, value: returned_variable };
        ReturnSplit { declaration, substitution, final_return }
    } else {
        unreachable!()
    }
}

fn into_single_substitution(stmt: Statement, stmts: &mut Vec<Statement>) {
    use Statement::Substitution;
    use num_bigint_dig::BigInt;
    use Expression::{ArrayInLine, Number, TernaryOp};
    use Statement::{Block, IfThenElse};

    match stmt {
        Substitution { var, access, op, rhe: TernaryOp { cond, if_true, if_false, .. }, meta } => {
            let mut if_assigns = vec![];
            let sub_if = Substitution {
                meta: meta.clone(),
                var: var.clone(),
                access: access.clone(),
                op: op.clone(),
                rhe: *if_true,
            };
            if_assigns.push(sub_if);

            let mut else_assigns = vec![];
            let sub_else = Substitution { op, var, access, meta: meta.clone(), rhe: *if_false };
            else_assigns.push(sub_else);

            let if_body = Block { stmts: if_assigns, meta: meta.clone() };
            let else_body = Block { stmts: else_assigns, meta: meta.clone() };
            let stmt = IfThenElse {
                meta,
                cond: *cond,
                if_case: Box::new(if_body),
                else_case: Option::Some(Box::new(else_body)),
            };

            stmts.push(stmt);
        }
        Substitution { var, access, op, rhe: ArrayInLine { values, .. }, meta } => {
            for (index, v) in values.into_iter().enumerate() {
                let mut index_meta = meta.clone();
                index_meta.get_mut_memory_knowledge().set_concrete_dimensions(vec![]);
                let expr_index = Number { meta: index_meta, value: BigInt::from(index) };
                let as_access = Access::ArrayAccess(expr_index);
                let mut accessed_with = access.clone();
                accessed_with.push(as_access);
                let sub = Substitution {
                    op,
                    var: var.clone(),
                    access: accessed_with,
                    meta: meta.clone(),
                    rhe: v,
                };
                stmts.push(sub);
            }
        }
        _ => stmts.push(stmt),
    }
}
