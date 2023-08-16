use dada_ir::{
    code::{bir, validated::op::Op},
    error,
};

use crate::{
    error::DiagnosticBuilderExt,
    machine::op::MachineOpExtMut,
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
                Op::EqualEqual => Ok(self.machine.our_value(self.machine.pc(), lhs == rhs)),
                Op::GreaterEqual => Ok(self.machine.our_value(self.machine.pc(), lhs >= rhs)),
                Op::LessEqual => Ok(self.machine.our_value(self.machine.pc(), lhs <= rhs)),
                _ => op_error(),
            },
            (&ObjectData::UnsignedInt(lhs), &ObjectData::UnsignedInt(rhs))
            | (&ObjectData::UnsignedInt(lhs), &ObjectData::Int(rhs))
            | (&ObjectData::Int(lhs), &ObjectData::UnsignedInt(rhs)) => match op {
                Op::EqualEqual => Ok(self.machine.our_value(self.machine.pc(), lhs == rhs)),
                Op::GreaterEqual => Ok(self.machine.our_value(self.machine.pc(), lhs >= rhs)),
                Op::LessEqual => Ok(self.machine.our_value(self.machine.pc(), lhs <= rhs)),
                Op::Plus => match lhs.checked_add(rhs) {
                    Some(value) => Ok(self.machine.our_value(self.machine.pc(), value)),
                    None => overflow_error(),
                },
                Op::Minus => match lhs.checked_sub(rhs) {
                    Some(value) => Ok(self.machine.our_value(self.machine.pc(), value)),
                    None => overflow_error(),
                },
                Op::Times => match lhs.checked_mul(rhs) {
                    Some(value) => Ok(self.machine.our_value(self.machine.pc(), value)),
                    None => overflow_error(),
                },
                Op::DividedBy => match lhs.checked_div(rhs) {
                    Some(value) => Ok(self.machine.our_value(self.machine.pc(), value)),
                    None => div_zero_error(),
                },
                Op::LessThan => Ok(self.machine.our_value(self.machine.pc(), lhs < rhs)),
                Op::GreaterThan => Ok(self.machine.our_value(self.machine.pc(), lhs > rhs)),
            },
            (&ObjectData::Int(lhs), &ObjectData::Int(rhs)) => match op {
                Op::EqualEqual => Ok(self.machine.our_value(self.machine.pc(), lhs == rhs)),
                Op::GreaterEqual => Ok(self.machine.our_value(self.machine.pc(), lhs >= rhs)),
                Op::LessEqual => Ok(self.machine.our_value(self.machine.pc(), lhs <= rhs)),
                Op::Plus => match lhs.checked_add(rhs) {
                    Some(value) => Ok(self
                        .machine
                        .our_value(self.machine.pc(), ObjectData::Int(value))),
                    None => overflow_error(),
                },
                Op::Minus => match lhs.checked_sub(rhs) {
                    Some(value) => Ok(self
                        .machine
                        .our_value(self.machine.pc(), ObjectData::Int(value))),
                    None => overflow_error(),
                },
                Op::Times => match lhs.checked_mul(rhs) {
                    Some(value) => Ok(self
                        .machine
                        .our_value(self.machine.pc(), ObjectData::Int(value))),
                    None => overflow_error(),
                },
                Op::DividedBy => match lhs.checked_div(rhs) {
                    Some(value) => Ok(self
                        .machine
                        .our_value(self.machine.pc(), ObjectData::Int(value))),
                    None => div_zero_error(),
                },
                Op::LessThan => Ok(self.machine.our_value(self.machine.pc(), lhs < rhs)),
                Op::GreaterThan => Ok(self.machine.our_value(self.machine.pc(), lhs > rhs)),
            },
            (&ObjectData::SignedInt(lhs), &ObjectData::SignedInt(rhs)) => {
                self.apply_signed_int(expr, op, lhs, rhs)
            }
            (&ObjectData::Int(lhs), &ObjectData::SignedInt(rhs)) => match i64::try_from(lhs) {
                Ok(lhs) => self.apply_signed_int(expr, op, lhs, rhs),
                Err(_) => overflow_error(),
            },
            (&ObjectData::SignedInt(lhs), &ObjectData::Int(rhs)) => match i64::try_from(rhs) {
                Ok(rhs) => self.apply_signed_int(expr, op, lhs, rhs),
                Err(_) => overflow_error(),
            },
            (&ObjectData::Float(lhs), &ObjectData::Float(rhs)) => match op {
                Op::EqualEqual => Ok(self.machine.our_value(self.machine.pc(), lhs == rhs)),
                Op::GreaterEqual => Ok(self.machine.our_value(self.machine.pc(), lhs >= rhs)),
                Op::LessEqual => Ok(self.machine.our_value(self.machine.pc(), lhs <= rhs)),
                Op::Plus => Ok(self.machine.our_value(self.machine.pc(), lhs + rhs)),
                Op::Minus => Ok(self.machine.our_value(self.machine.pc(), lhs - rhs)),
                Op::Times => Ok(self.machine.our_value(self.machine.pc(), lhs * rhs)),
                Op::DividedBy => Ok(self.machine.our_value(self.machine.pc(), lhs / rhs)),
                Op::LessThan => Ok(self.machine.our_value(self.machine.pc(), lhs < rhs)),
                Op::GreaterThan => Ok(self.machine.our_value(self.machine.pc(), lhs > rhs)),
            },
            (ObjectData::String(lhs), ObjectData::String(rhs)) => match op {
                Op::EqualEqual => {
                    let val = lhs == rhs;
                    Ok(self.machine.our_value(self.machine.pc(), val))
                }
                Op::GreaterEqual => {
                    let val = lhs >= rhs;
                    Ok(self.machine.our_value(self.machine.pc(), val))
                }
                Op::LessEqual => {
                    let val = lhs <= rhs;
                    Ok(self.machine.our_value(self.machine.pc(), val))
                }
                _ => op_error(),
            },
            (&ObjectData::Unit(()), &ObjectData::Unit(())) => match op {
                Op::EqualEqual => Ok(self.machine.our_value(self.machine.pc(), true)),
                Op::GreaterEqual => Ok(self.machine.our_value(self.machine.pc(), lhs >= rhs)),
                Op::LessEqual => Ok(self.machine.our_value(self.machine.pc(), lhs <= rhs)),
                _ => op_error(),
            },
            _ => op_error(),
        }
    }

    fn apply_signed_int(
        &mut self,
        expr: bir::Expr,
        op: Op,
        lhs: i64,
        rhs: i64,
    ) -> eyre::Result<Value> {
        let div_zero_error = || {
            let span = self.span_from_bir(expr);
            Err(error!(span, "divide by zero").eyre(self.db))
        };
        let overflow_error = || {
            let span = self.span_from_bir(expr);
            Err(error!(span, "overflow").eyre(self.db))
        };
        match op {
            Op::EqualEqual => Ok(self.machine.our_value(self.machine.pc(), lhs == rhs)),
            Op::GreaterEqual => Ok(self.machine.our_value(self.machine.pc(), lhs >= rhs)),
            Op::LessEqual => Ok(self.machine.our_value(self.machine.pc(), lhs <= rhs)),
            Op::Plus => match lhs.checked_add(rhs) {
                Some(value) => Ok(self.machine.our_value(self.machine.pc(), value)),
                None => overflow_error(),
            },
            Op::Minus => match lhs.checked_sub(rhs) {
                Some(value) => Ok(self.machine.our_value(self.machine.pc(), value)),
                None => overflow_error(),
            },
            Op::Times => match lhs.checked_mul(rhs) {
                Some(value) => Ok(self.machine.our_value(self.machine.pc(), value)),
                None => overflow_error(),
            },
            Op::DividedBy => match lhs.checked_div(rhs) {
                Some(value) => Ok(self.machine.our_value(self.machine.pc(), value)),
                None => {
                    if rhs != -1 {
                        div_zero_error()
                    } else {
                        let span = self.span_from_bir(expr);
                        Err(error!(span, "signed division overflow").eyre(self.db))
                    }
                }
            },
            Op::LessThan => Ok(self.machine.our_value(self.machine.pc(), lhs < rhs)),
            Op::GreaterThan => Ok(self.machine.our_value(self.machine.pc(), lhs > rhs)),
        }
    }
}
