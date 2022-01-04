use std::{future::Future, pin::Pin};

use dada_collections::Map;
use dada_id::prelude::*;
use dada_ir::{
    code::{bir, syntax},
    diagnostic::Fallible,
    error,
    origin_table::HasOriginIn,
    span::FileSpan,
};

use crate::{data::Tuple, interpreter::Interpreter, value::Value};

struct StackFrame<'me> {
    interpreter: &'me Interpreter<'me>,
    bir: bir::Bir,
    tables: &'me bir::Tables,
    local_variables: Map<bir::LocalVariable, Value>,
}

impl Interpreter<'_> {
    pub(crate) fn execute_bir<'me>(
        &'me self,
        bir: bir::Bir,
    ) -> Pin<Box<dyn Future<Output = Fallible<Value>> + 'me>> {
        let bir::BirData {
            tables,
            start_basic_block,
        } = bir.data(self.db());
        let stack_frame = StackFrame {
            interpreter: self,
            bir,
            tables,
            local_variables: Map::default(),
        };
        Box::pin(stack_frame.execute(*start_basic_block))
    }
}

impl StackFrame<'_> {
    fn db(&self) -> &dyn crate::Db {
        self.interpreter.db()
    }

    async fn execute(mut self, mut basic_block: bir::BasicBlock) -> Fallible<Value> {
        let interpreter = self.interpreter;
        loop {
            let basic_block_data = basic_block.data(self.tables);
            for statement in &basic_block_data.statements {
                match statement.data(self.tables) {
                    dada_ir::code::bir::StatementData::Assign(place, expr) => {
                        let expr_value = self.evaluate_bir_expr(*expr)?;
                        self.assign_place(*place, expr_value).await;
                    }
                }
            }

            match basic_block_data.terminator.data(self.tables) {
                dada_ir::code::bir::TerminatorData::Goto(next_block) => {
                    basic_block = *next_block;
                }
                dada_ir::code::bir::TerminatorData::If(place, if_true, if_false) => {
                    if self.eval_place_to_bool(*place)? {
                        basic_block = *if_true;
                    } else {
                        basic_block = *if_false;
                    }
                }
                dada_ir::code::bir::TerminatorData::StartAtomic(next_block) => {
                    basic_block = *next_block;
                }
                dada_ir::code::bir::TerminatorData::EndAtomic(next_block) => {
                    basic_block = *next_block;
                }
                dada_ir::code::bir::TerminatorData::Return(place) => {
                    return self.give_place(*place);
                }
                dada_ir::code::bir::TerminatorData::Assign(place, expr, next) => {
                    let value = self.evaluate_terminator_expr(expr).await?;
                    self.assign_place(*place, value);
                    basic_block = *next;
                }
                dada_ir::code::bir::TerminatorData::Error => {
                    let span = self.span_from_bir(basic_block_data.terminator);
                    return Err(error!(span, "compilation error").emit(self.db()));
                }
                dada_ir::code::bir::TerminatorData::Panic => {
                    let span = self.span_from_bir(basic_block_data.terminator);
                    return Err(error!(span, "panic").emit(self.db()));
                }
            }
        }
    }

    fn evaluate_bir_expr(&mut self, expr: bir::Expr) -> Fallible<Value> {
        match expr.data(self.tables) {
            bir::ExprData::BooleanLiteral(value) => Ok(Value::new(self.interpreter, *value)),
            bir::ExprData::IntegerLiteral(value) => Ok(Value::new(self.interpreter, *value)),
            bir::ExprData::StringLiteral(value) => Ok(Value::new(self.interpreter, *value)),
            bir::ExprData::ShareValue(expr) => {
                self.evaluate_bir_expr(*expr)?.into_share(self.interpreter)
            }
            bir::ExprData::Share(place) => self.with_place(*place, Value::share),
            bir::ExprData::Lease(place) => self.with_place(*place, Value::lease),
            bir::ExprData::Give(place) => self.with_place(*place, Value::give),
            bir::ExprData::Tuple(places) => {
                let fields = places
                    .iter()
                    .map(|place| self.give_place(*place))
                    .collect::<Fallible<Vec<_>>>()?;
                Ok(Value::new(self.interpreter, Tuple { fields }))
            }
            bir::ExprData::Op(lhs, op, rhs) => todo!(),
            bir::ExprData::Error => {
                let span = self.span_from_bir(expr);
                Err(error!(span, "compilation error").emit(self.db()))
            }
        }
    }

    fn give_place<'s>(&'s mut self, place: bir::Place) -> Fallible<Value> {
        self.with_place(place, Value::give)
    }

    fn span_from_bir(
        &self,
        expr: impl HasOriginIn<bir::Origins, Origin = syntax::Expr>,
    ) -> FileSpan {
        self.interpreter.span_from_bir(self.bir, expr)
    }

    async fn assign_place(&mut self, place: bir::Place, value: Value) -> Fallible<()> {
        match place.data(self.tables) {
            bir::PlaceData::LocalVariable(local_variable) => {
                // FIXME: Presently infallible, but think about atomic etc eventually. =)
                let slot = self.local_variables.get_mut(local_variable).unwrap();
                *slot = value;
                Ok(())
            }
            bir::PlaceData::Function(function) => {
                let span_now = self.interpreter.span_now();
                let name = function.name(self.db()).as_str(self.db());
                let name_span = function.name_span(self.db());
                Err(error!(span_now, "cannot assign to `{}`", name)
                    .secondary_label(
                        name_span,
                        &format!("`{}` is a function, declared here", name),
                    )
                    .emit(self.db()))
            }
            bir::PlaceData::Class(class) => {
                let span_now = self.interpreter.span_now();
                let name = class.name(self.db()).as_str(self.db());
                let name_span = class.name_span(self.db());
                Err(error!(span_now, "cannot assign to `{}`", name)
                    .secondary_label(name_span, &format!("`{}` is a class, declared here", name))
                    .emit(self.db()))
            }
            bir::PlaceData::Intrinsic(intrinsic) => {
                let span_now = self.interpreter.span_now();
                let name = intrinsic.as_str(self.db());
                Err(error!(span_now, "cannot assign to `{}`", name).emit(self.db()))
            }
            bir::PlaceData::Dot(owner_place, field_name) => {
                self.with_place(*owner_place, |owner_value, interpreter| {
                    owner_value.write(interpreter, |data| data.assign_field(*field_name, value))
                })
            }
        }
    }

    fn with_place<R>(
        &mut self,
        place: bir::Place,
        op: impl FnOnce(&Value, &Interpreter) -> Fallible<R>,
    ) -> Fallible<R> {
        match place.data(self.tables) {
            bir::PlaceData::LocalVariable(local_variable) => op(
                self.local_variables.get(local_variable).unwrap(),
                self.interpreter,
            ),
            bir::PlaceData::Function(function) => {
                op(&Value::new(self.interpreter, *function), self.interpreter)
            }
            bir::PlaceData::Class(class) => {
                op(&Value::new(self.interpreter, *class), self.interpreter)
            }
            bir::PlaceData::Intrinsic(intrinsic) => {
                op(&Value::new(self.interpreter, *intrinsic), self.interpreter)
            }
            bir::PlaceData::Dot(place, word) => self.with_place(*place, |value, interpreter| {
                value.field(interpreter, *word, |v| op(v, interpreter))
            }),
        }
    }

    fn eval_place_to_bool(&mut self, place: bir::Place) -> Fallible<bool> {
        self.with_place(place, |value, interpreter| {
            value.read(interpreter, |data| data.to_bool(interpreter))
        })
    }

    async fn evaluate_terminator_expr(&self, expr: &bir::TerminatorExpr) -> Fallible<Value> {
        todo!()
    }
}
