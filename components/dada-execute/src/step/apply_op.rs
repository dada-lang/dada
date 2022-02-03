use dada_ir::{
    code::{bir, validated::op::Op},
    error,
};

use crate::{
    error::DiagnosticBuilderExt,
    machine::op::MachineOpExt,
    machine::{Object, ObjectData, Value},
};

use super::Stepper;

impl Stepper<'_> {
    pub(super) fn apply_op(
        &mut self,
        expr: bir::Expr,
        op: Op,
        lhs: Object,
        rhs: Object,
    ) -> eyre::Result<Value> {
        let op_error = || {
            let span = self.span_from_bir(expr);
            Err(error!(
                span,
                "cannot apply operator {} to {} and {}",
                op,
                self.machine[lhs].kind_str(self.db),
                self.machine[rhs].kind_str(self.db),
            )
            .eyre(self.db))
        };
        let div_zero_error = || {
            let span = self.span_from_bir(expr);
            Err(error!(span, "divide by zero").eyre(self.db))
        };
        let overflow_error = || {
            let span = self.span_from_bir(expr);
            Err(error!(span, "overflow").eyre(self.db))
        };
        match (&self.machine[lhs], &self.machine[rhs]) {
            (&ObjectData::Bool(lhs), &ObjectData::Bool(rhs)) => match op {
                Op::EqualEqual => Ok(self.machine.our_value(lhs == rhs)),
                _ => op_error(),
            },
            (&ObjectData::Uint(lhs), &ObjectData::Uint(rhs)) => match op {
                Op::EqualEqual => Ok(self.machine.our_value(lhs == rhs)),
                Op::Plus => match lhs.checked_add(rhs) {
                    Some(value) => Ok(self.machine.our_value(value)),
                    None => overflow_error(),
                },
                Op::Minus => match lhs.checked_sub(rhs) {
                    Some(value) => Ok(self.machine.our_value(value)),
                    None => overflow_error(),
                },
                Op::Times => match lhs.checked_mul(rhs) {
                    Some(value) => Ok(self.machine.our_value(value)),
                    None => overflow_error(),
                },
                Op::DividedBy => match lhs.checked_div(rhs) {
                    Some(value) => Ok(self.machine.our_value(value)),
                    None => div_zero_error(),
                },
                Op::LessThan => Ok(self.machine.our_value(lhs < rhs)),
                Op::GreaterThan => Ok(self.machine.our_value(lhs > rhs)),
            },
            (&ObjectData::Int(lhs), &ObjectData::Int(rhs)) => match op {
                Op::EqualEqual => Ok(self.machine.our_value(lhs == rhs)),
                Op::Plus => match lhs.checked_add(rhs) {
                    Some(value) => Ok(self.machine.our_value(value)),
                    None => overflow_error(),
                },
                Op::Minus => match lhs.checked_sub(rhs) {
                    Some(value) => Ok(self.machine.our_value(value)),
                    None => overflow_error(),
                },
                Op::Times => match lhs.checked_mul(rhs) {
                    Some(value) => Ok(self.machine.our_value(value)),
                    None => overflow_error(),
                },
                Op::DividedBy => match lhs.checked_div(rhs) {
                    Some(value) => Ok(self.machine.our_value(value)),
                    None => {
                        if rhs != -1 {
                            div_zero_error()
                        } else {
                            let span = self.span_from_bir(expr);
                            Err(error!(span, "signed division overflow").eyre(self.db))
                        }
                    }
                },
                Op::LessThan => Ok(self.machine.our_value(lhs < rhs)),
                Op::GreaterThan => Ok(self.machine.our_value(lhs > rhs)),
            },
            (&ObjectData::String(lhs), &ObjectData::String(rhs)) => match op {
                Op::EqualEqual => Ok(self.machine.our_value(lhs == rhs)),
                _ => op_error(),
            },
            (&ObjectData::Unit(()), &ObjectData::Unit(())) => match op {
                Op::EqualEqual => Ok(self.machine.our_value(true)),
                _ => op_error(),
            },
            _ => op_error(),
        }
    }
}
