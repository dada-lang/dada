use std::{future::Future, pin::Pin};

use dada_collections::Map;
use dada_id::prelude::*;
use dada_ir::code::bir;

use crate::{
    interpreter::{Interpreter, StackFrameClock},
    value::Value,
};

struct StackFrame<'me> {
    interpreter: &'me Interpreter<'me>,
    tables: &'me bir::Tables,
    local_variables: Map<bir::LocalVariable, Value>,
}

impl Interpreter<'_> {
    pub fn execute_bir<'me>(
        &'me self,
        bir: bir::Bir,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<Value>> + 'me>> {
        let bir::BirData {
            tables,
            start_basic_block,
        } = bir.data(self.db());
        let stack_frame = StackFrame {
            interpreter: self,
            tables,
            local_variables: Map::default(),
        };
        Box::pin(stack_frame.execute(*start_basic_block))
    }
}

impl StackFrame<'_> {
    async fn execute(mut self, mut basic_block: bir::BasicBlock) -> eyre::Result<Value> {
        loop {
            let basic_block_data = basic_block.data(self.tables);
            for statement in &basic_block_data.statements {
                match statement.data(self.tables) {
                    dada_ir::code::bir::StatementData::Assign(place, expr) => {
                        let expr_value = self.evaluate_bir_expr(*expr).await?;
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
                    let value = self.eval_place(*place)?;
                    return Ok(value);
                }
                dada_ir::code::bir::TerminatorData::Assign(place, expr, next) => {}
                dada_ir::code::bir::TerminatorData::Error => {
                    return Err(eyre::eyre!("compilation error")); //FIXME
                }
                dada_ir::code::bir::TerminatorData::Panic => {
                    return Err(eyre::eyre!("panic")); //FIXME
                }
            }
        }
    }

    async fn evaluate_bir_expr(&mut self, expr: bir::Expr) -> eyre::Result<Value> {
        match expr.data(self.tables) {
            bir::ExprData::Place(place) => self.eval_place(*place),
            bir::ExprData::BooleanLiteral(_) => todo!(),
            bir::ExprData::IntegerLiteral(_) => todo!(),
            bir::ExprData::StringLiteral(_) => todo!(),
            bir::ExprData::Share(_) => todo!(),
            bir::ExprData::Lease(_) => todo!(),
            bir::ExprData::Give(_) => todo!(),
            bir::ExprData::Tuple(places) => todo!(),
            bir::ExprData::New(class, args) => todo!(),
            bir::ExprData::Op(lhs, op, rhs) => todo!(),
            bir::ExprData::Error => Err(eyre::eyre!("compilation error")),
        }
    }

    async fn assign_place(&mut self, place: bir::Place, value: Value) -> eyre::Result<()> {
        todo!()
    }

    fn eval_place(&mut self, place: bir::Place) -> eyre::Result<Value> {
        match place.data(self.tables) {
            bir::PlaceData::LocalVariable(local_variable) => {
                Ok(self.local_variables.get(local_variable).unwrap().clone())
            }
            bir::PlaceData::Function(function) => {}
            bir::PlaceData::Class(class) => todo!(),
            bir::PlaceData::Intrinsic(intrinsic) => todo!(),
            bir::PlaceData::Dot(place, word) => {
                let value = self.eval_place(*place)?;
                Ok(value.field(*word))
            }
        }
    }

    fn eval_place_to_bool(&mut self, place: bir::Place) -> eyre::Result<bool> {
        todo!()
    }
}
