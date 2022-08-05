use crate::*;

impl Expression {
    pub fn get_meta(&self) -> &Meta {
        use Expression::*;
        match self {
            InfixOp { meta, .. }
            | PrefixOp { meta, .. }
            | TernaryOp { meta, .. }
            | Variable { meta, .. }
            | Number { meta, .. }
            | Call { meta, .. }
            | ArrayInLine { meta, .. } => meta,
        }
    }

    pub fn get_mut_meta(&mut self) -> &mut Meta {
        use Expression::*;
        match self {
            InfixOp { meta, .. }
            | PrefixOp { meta, .. }
            | TernaryOp { meta, .. }
            | Variable { meta, .. }
            | Number { meta, .. }
            | Call { meta, .. }
            | ArrayInLine { meta, .. } => meta,
        }
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Expression::ArrayInLine { .. })
    }

    pub fn is_infix(&self) -> bool {
        matches!(self, Expression::InfixOp { .. })
    }

    pub fn is_prefix(&self) -> bool {
        matches!(self, Expression::PrefixOp { .. })
    }

    pub fn is_switch(&self) -> bool {
        matches!(self, Expression::TernaryOp { .. })
    }

    pub fn is_variable(&self) -> bool {
        matches!(self, Expression::Variable { .. })
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Expression::Number { .. })
    }

    pub fn is_call(&self) -> bool {
        matches!(self, Expression::Call { .. })
    }
}

impl FillMeta for Expression {
    fn fill(&mut self, file_id: usize, elem_id: &mut usize) {
        use Expression::*;
        
        self.get_mut_meta().elem_id = *elem_id;
        *elem_id += 1;

        self.get_mut_meta().set_file_id(file_id);

        match self {
            Number { .. } => (),
            Variable { access, .. } => {
                for acc in access {
                    if let Access::ArrayAccess(e) = acc {
                        e.fill(file_id, elem_id)
                    }
                }
            }
            InfixOp { lhe, rhe, .. } => {
                lhe.fill(file_id, elem_id);
                rhe.fill(file_id, elem_id);
            }
            PrefixOp { rhe, .. } => {
                rhe.fill(file_id, elem_id);
            }
            TernaryOp { cond, if_false, if_true, .. } => {
                cond.fill(file_id, elem_id);
                if_true.fill(file_id, elem_id);
                if_false.fill(file_id, elem_id);
            }
            Call { args, .. } => {
                args.iter_mut().for_each(|a| a.fill(file_id, elem_id));
            }
            ArrayInLine { values, .. } => {
                values.iter_mut().for_each(|v| v.fill(file_id, elem_id));
            }
        }
    }
}
