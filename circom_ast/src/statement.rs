use serde_derive::{Deserialize, Serialize};

use crate::{Expression, Meta, FillMeta};

#[derive(Clone)]
pub enum Statement {
    IfThenElse {
        meta: Meta,
        cond: Expression,
        if_case: Box<Statement>,
        else_case: Option<Box<Statement>>,
    },
    While {
        meta: Meta,
        cond: Expression,
        stmt: Box<Statement>,
    },
    Return {
        meta: Meta,
        value: Expression,
    },
    InitializationBlock {
        meta: Meta,
        xtype: VariableType,
        initializations: Vec<Statement>,
    },
    Declaration {
        meta: Meta,
        xtype: VariableType,
        name: String,
        dimensions: Vec<Expression>,
        is_constant: bool,
    },
    Substitution {
        meta: Meta,
        var: String,
        access: Vec<Access>,
        op: AssignOp,
        rhe: Expression,
    },
    ConstraintEquality {
        meta: Meta,
        lhe: Expression,
        rhe: Expression,
    },
    LogCall {
        meta: Meta,
        arg: Expression,
    },
    Block {
        meta: Meta,
        stmts: Vec<Statement>,
    },
    Assert {
        meta: Meta,
        arg: Expression,
    },
}

impl Statement {
    pub fn build_conditional_block(
        meta: Meta,
        cond: Expression,
        if_case: Statement,
        else_case: Option<Statement>,
    ) -> Statement {
        Statement::IfThenElse {
            meta,
            cond,
            else_case: else_case.map(Box::new),
            if_case: Box::new(if_case),
        }
    }

    pub fn build_while_block(meta: Meta, cond: Expression, stmt: Statement) -> Statement {
        Statement::While { meta, cond, stmt: Box::new(stmt) }
    }

    pub fn build_initialization_block(
        meta: Meta,
        xtype: VariableType,
        initializations: Vec<Statement>,
    ) -> Statement {
        Statement::InitializationBlock { meta, xtype, initializations }
    }
    pub fn build_block(meta: Meta, stmts: Vec<Statement>) -> Statement {
        Statement::Block { meta, stmts }
    }

    pub fn build_return(meta: Meta, value: Expression) -> Statement {
        Statement::Return { meta, value }
    }

    pub fn build_declaration(
        meta: Meta,
        xtype: VariableType,
        name: String,
        dimensions: Vec<Expression>,
    ) -> Statement {
        let is_constant = true;
        Statement::Declaration { meta, xtype, name, dimensions, is_constant }
    }

    pub fn build_substitution(
        meta: Meta,
        var: String,
        access: Vec<Access>,
        op: AssignOp,
        rhe: Expression,
    ) -> Statement {
        Statement::Substitution { meta, var, access, op, rhe }
    }

    pub fn build_constraint_equality(meta: Meta, lhe: Expression, rhe: Expression) -> Statement {
        Statement::ConstraintEquality { meta, lhe, rhe }
    }

    pub fn build_log_call(meta: Meta, arg: Expression) -> Statement {
        Statement::LogCall { meta, arg }
    }

    pub fn build_assert(meta: Meta, arg: Expression) -> Statement {
        Statement::Assert { meta, arg }
    }

    pub fn get_meta(&self) -> &Meta {
        use Statement::*;
        match self {
            IfThenElse { meta, .. }
            | While { meta, .. }
            | Return { meta, .. }
            | Declaration { meta, .. }
            | Substitution { meta, .. }
            | LogCall { meta, .. }
            | Block { meta, .. }
            | Assert { meta, .. }
            | ConstraintEquality { meta, .. }
            | InitializationBlock { meta, .. } => meta,
        }
    }
    pub fn get_mut_meta(&mut self) -> &mut Meta {
        use Statement::*;
        match self {
            IfThenElse { meta, .. }
            | While { meta, .. }
            | Return { meta, .. }
            | Declaration { meta, .. }
            | Substitution { meta, .. }
            | LogCall { meta, .. }
            | Block { meta, .. }
            | Assert { meta, .. }
            | ConstraintEquality { meta, .. }
            | InitializationBlock { meta, .. } => meta,
        }
    }

    pub fn is_if_then_else(&self) -> bool {
        matches!(self, Statement::IfThenElse { .. })
    }

    pub fn is_while(&self) -> bool {
        matches!(self, Statement::While { .. })
    }

    pub fn is_return(&self) -> bool {
        use Statement::Return;
        matches!(self, Return { .. })
    }

    pub fn is_initialization_block(&self) -> bool {
        matches!(self, Statement::InitializationBlock { .. })
    }

    pub fn is_declaration(&self) -> bool {
        matches!(self, Statement::Declaration { .. })
    }

    pub fn is_substitution(&self) -> bool {
        matches!(self, Statement::Substitution { .. })
    }

    pub fn is_constraint_equality(&self) -> bool {
        matches!(self, Statement::ConstraintEquality { .. })
    }

    pub fn is_log_call(&self) -> bool {
        matches!(self, Statement::LogCall { .. })
    }

    pub fn is_block(&self) -> bool {
        matches!(self, Statement::Block { .. })
    }

    pub fn is_assert(&self) -> bool {
        matches!(self, Statement::Assert { .. })
    }
}

impl FillMeta for Statement {
    fn fill(&mut self, file_id: usize, element_id: &mut usize) {
        use Statement::*;
        self.get_mut_meta().elem_id = *element_id;
        *element_id += 1;

        self.get_mut_meta().set_file_id(file_id);

        match self {
            IfThenElse { cond, if_case, else_case, .. } => {
                cond.fill(file_id, element_id);
                if_case.fill(file_id, element_id);
                if let Some(s) = else_case {
                    s.fill(file_id, element_id);
                }
            }
            While { cond, stmt, .. } => {
                cond.fill(file_id, element_id);
                stmt.fill(file_id, element_id);
            }
            Return { value, .. } => {
                value.fill(file_id, element_id);
            }
            InitializationBlock { initializations, .. } => {
                initializations.iter_mut().for_each(|init| init.fill(file_id, element_id));
            }
            Declaration { dimensions, .. } => {
                dimensions.iter_mut().for_each(|d| d.fill(file_id, element_id));
            }
            Substitution { access, rhe, .. } => {
                rhe.fill(file_id, element_id);
                for a in access {
                    if let Access::ArrayAccess(e) = a {
                        e.fill(file_id, element_id);
                    }
                }
            }
            ConstraintEquality { lhe, rhe, .. } => {
                lhe.fill(file_id, element_id);
                rhe.fill(file_id, element_id);
            }
            LogCall { arg, .. } => {
                arg.fill(file_id, element_id);
            }
            Block { stmts, .. } => {
                stmts.iter_mut().for_each(|s| s.fill(file_id, element_id));
            }
            Assert { arg, .. } => {
                arg.fill(file_id, element_id);
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SignalElementType {
    Empty,
    Binary,
    FieldElement,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SignalType {
    Output,
    Input,
    Intermediate,
}

#[derive(Copy, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub enum VariableType {
    Var,
    Signal(SignalType, SignalElementType),
    Component,
}

#[derive(Clone)]
pub enum Access {
    ComponentAccess(String),
    ArrayAccess(Expression),
}

impl Access {
    pub fn build_component_access(acc: String) -> Access {
        Access::ComponentAccess(acc)
    }
    pub fn build_array_access(expr: Expression) -> Access {
        Access::ArrayAccess(expr)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum AssignOp {
    AssignVar,
    AssignSignal,
    AssignConstraintSignal,
}

impl AssignOp {
    pub fn is_signal_operator(self) -> bool {
        matches!(self, AssignOp::AssignConstraintSignal | AssignOp::AssignSignal)
    }
}
