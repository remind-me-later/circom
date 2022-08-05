use serde_derive::{Deserialize, Serialize};

use crate::{Expression, Meta};

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
