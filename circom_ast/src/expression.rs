use num_bigint_dig::BigInt;

use crate::{Meta, Access};

#[derive(Clone)]
pub enum Expression {
    InfixOp {
        meta: Meta,
        lhe: Box<Expression>,
        infix_op: ExpressionInfixOpcode,
        rhe: Box<Expression>,
    },
    PrefixOp {
        meta: Meta,
        prefix_op: ExpressionPrefixOpcode,
        rhe: Box<Expression>,
    },
    TernaryOp {
        meta: Meta,
        cond: Box<Expression>,
        if_true: Box<Expression>,
        if_false: Box<Expression>,
    },
    Variable {
        meta: Meta,
        name: String,
        access: Vec<Access>,
    },
    Number {
        meta: Meta,
        value: BigInt,
    },
    Call {
        meta: Meta,
        id: String,
        args: Vec<Expression>,
    },
    ArrayInLine {
        meta: Meta,
        values: Vec<Expression>,
    },
}

impl Expression {
    pub fn build_infix(
        meta: Meta,
        lhe: Expression,
        infix_op: ExpressionInfixOpcode,
        rhe: Expression,
    ) -> Expression {
        Expression::InfixOp { meta, infix_op, lhe: Box::new(lhe), rhe: Box::new(rhe) }
    }

    pub fn build_prefix(
        meta: Meta,
        prefix_op: ExpressionPrefixOpcode,
        rhe: Expression,
    ) -> Expression {
        Expression::PrefixOp { meta, prefix_op, rhe: Box::new(rhe) }
    }

    pub fn build_ternary_op(
        meta: Meta,
        cond: Expression,
        if_true: Expression,
        if_false: Expression,
    ) -> Expression {
        Expression::TernaryOp {
            meta,
            cond: Box::new(cond),
            if_true: Box::new(if_true),
            if_false: Box::new(if_false),
        }
    }

    pub fn build_variable(meta: Meta, name: String, access: Vec<Access>) -> Expression {
        Expression::Variable { meta, name, access }
    }

    pub fn build_number(meta: Meta, value: BigInt) -> Expression {
        Expression::Number { meta, value }
    }

    pub fn build_call(meta: Meta, id: String, args: Vec<Expression>) -> Expression {
        Expression::Call { meta, id, args }
    }

    pub fn build_array_in_line(meta: Meta, values: Vec<Expression>) -> Expression {
        Expression::ArrayInLine { meta, values }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum ExpressionInfixOpcode {
    Mul,
    Div,
    Add,
    Sub,
    Pow,
    IntDiv,
    Mod,
    ShiftL,
    ShiftR,
    LesserEq,
    GreaterEq,
    Lesser,
    Greater,
    Eq,
    NotEq,
    BoolOr,
    BoolAnd,
    BitOr,
    BitAnd,
    BitXor,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ExpressionPrefixOpcode {
    Sub,
    BoolNot,
    Complement,
}
