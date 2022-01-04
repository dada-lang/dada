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
use dada_parse::prelude::*;

use crate::{
    data::{Data, Tuple},
    interpreter::Interpreter,
    value::Value,
};

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
            bir::ExprData::Share(place) => {
                self.with_place(*place, |interpreter, value| value.share(interpreter))
            }
            bir::ExprData::Lease(place) => {
                self.with_place(*place, |interpreter, value| value.lease(interpreter))
            }

            bir::ExprData::Give(place) => {
                self.with_place(*place, |interpreter, value| value.give(interpreter))
            }
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
        self.with_place(place, |interpreter, value| value.give(interpreter))
    }

    fn span_from_bir(
        &self,
        expr: impl HasOriginIn<bir::Origins, Origin = syntax::Expr>,
    ) -> FileSpan {
        self.interpreter.span_from_bir(self.bir, expr)
    }

    async fn assign_place(&mut self, place: bir::Place, value: Value) -> Fallible<()> {
        todo!()
    }

    fn with_place<R>(
        &mut self,
        place: bir::Place,
        op: impl FnOnce(&Interpreter, &Value) -> Fallible<R>,
    ) -> Fallible<R> {
        match place.data(self.tables) {
            bir::PlaceData::LocalVariable(local_variable) => op(
                self.interpreter,
                self.local_variables.get(local_variable).unwrap(),
            ),
            bir::PlaceData::Function(function) => {
                op(self.interpreter, &Value::new(self.interpreter, *function))
            }
            bir::PlaceData::Class(class) => {
                op(self.interpreter, &Value::new(self.interpreter, *class))
            }
            bir::PlaceData::Intrinsic(intrinsic) => {
                op(self.interpreter, &Value::new(self.interpreter, *intrinsic))
            }
            bir::PlaceData::Dot(place, word) => self.with_place(*place, |interpreter, value| {
                value.field(interpreter, *word, |v| op(interpreter, v))
            }),
        }
    }

    fn eval_place_to_bool(&mut self, place: bir::Place) -> Fallible<bool> {
        self.with_place(place, |interpreter, value| {
            value.read(interpreter, |data| data.to_bool(interpreter))
        })
    }

    async fn evaluate_terminator_expr(&self, expr: &bir::TerminatorExpr) -> Fallible<Value> {
        todo!()
    }
}
