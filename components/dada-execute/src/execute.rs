use std::{future::Future, pin::Pin};

use dada_brew::prelude::*;
use dada_collections::IndexVec;
use dada_id::prelude::*;
use dada_ir::code::bir::LocalVariable;
use dada_ir::code::validated::op::Op;
use dada_ir::effect::Effect;
use dada_ir::function::Function;
use dada_ir::{
    code::{bir, syntax},
    error,
    origin_table::HasOriginIn,
    span::FileSpan,
};

use crate::kernel::Kernel;
use crate::thunk::Thunk;
use crate::{
    data::{Data, Tuple},
    error::DiagnosticBuilderExt,
    interpreter::Interpreter,
    value::Value,
};

/// Interprets a given function with the given kernel. Assumes this is the top stack frame.
pub async fn interpret(
    function: Function,
    db: &dyn crate::Db,
    kernel: &dyn Kernel,
    arguments: Vec<Value>,
) -> eyre::Result<()> {
    let initial_span = function.name_span(db);
    let interpreter = &Interpreter::new(db, kernel, initial_span);
    let bir = function.brew(db);
    let value = interpreter.execute_bir(bir, arguments, None).await?;
    value.read(interpreter, |data| data.to_unit(interpreter))
}

pub(crate) struct StackFrame<'me> {
    #[allow(dead_code)] // FIXME -- remove by end of PR
    parent_stack_frame: Option<&'me StackFrame<'me>>,
    interpreter: &'me Interpreter<'me>,
    bir: bir::Bir,
    tables: &'me bir::Tables,
    local_variables: IndexVec<bir::LocalVariable, Value>,
}

impl Interpreter<'_> {
    /// Executes the function on the given arguments. If function is an async
    /// function, this will simply return a thunk that, when awaited, runs the code.
    /// Otherwise it runs the function.
    pub(crate) fn execute_function(
        &self,
        function: Function,
        arguments: Vec<Value>,
        parent_stack_frame: Option<&StackFrame<'_>>,
    ) -> eyre::Result<Value> {
        if let Effect::Async = function.code(self.db()).effect {
            let thunk = Thunk::for_function(function, arguments);
            Ok(Value::new(self, thunk))
        } else {
            let bir = function.brew(self.db());
            let future = self.execute_bir(bir, arguments, parent_stack_frame);

            // This is interesting. `execute_bir` is async in *Rust* because it is
            // for both `fn` and `async fn` in Dada -- but in the case that we are
            // executing a Dada `fn`, we know that it will never await anything.
            // Therefore we can just poll it a single time.
            crate::poll_once::poll_once(future)
        }
    }

    /// Call and execute the code in the bir.
    ///
    /// Note that, if this bir comes from an async
    /// function, it will still just execute and doesn't return a thunk.
    /// This is convenient when calling `main`.
    pub(crate) fn execute_bir<'me>(
        &'me self,
        bir: bir::Bir,
        arguments: Vec<Value>,
        parent_stack_frame: Option<&'me StackFrame<'_>>,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<Value>> + 'me>> {
        let bir_data = bir.data(self.db());

        // Give each local variable an expired value with no permissions to start.
        let mut local_variables: IndexVec<bir::LocalVariable, Value> = bir_data
            .max_local_variable()
            .iter()
            .map(|_| Value::unit(self).give(self).unwrap())
            .collect();

        let num_parameters = bir_data.num_parameters;
        assert_eq!(
            num_parameters,
            arguments.len(),
            "wrong number of parameters provided"
        );
        for (local_variable, argument) in LocalVariable::range(0, num_parameters).zip(arguments) {
            local_variables.insert(local_variable, argument);
        }

        let stack_frame = StackFrame {
            interpreter: self,
            bir,
            tables: &bir_data.tables,
            local_variables,
            parent_stack_frame,
        };
        Box::pin(stack_frame.execute(bir_data.start_basic_block))
    }
}

impl StackFrame<'_> {
    fn db(&self) -> &dyn crate::Db {
        self.interpreter.db()
    }

    async fn execute(mut self, mut basic_block: bir::BasicBlock) -> eyre::Result<Value> {
        loop {
            let basic_block_data = basic_block.data(self.tables);
            for statement in &basic_block_data.statements {
                self.tick_clock(*statement);
                match statement.data(self.tables) {
                    dada_ir::code::bir::StatementData::Assign(place, expr) => {
                        let expr_value = self.evaluate_bir_expr(*expr)?;
                        self.assign_place(*place, expr_value)?;
                    }
                }
            }

            self.tick_clock(basic_block_data.terminator);
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
                    self.assign_place(*place, value)?;
                    basic_block = *next;
                }
                dada_ir::code::bir::TerminatorData::Error => {
                    let span = self.span_from_bir(basic_block_data.terminator);
                    return Err(error!(span, "compilation error").eyre(self.interpreter.db()));
                }
                dada_ir::code::bir::TerminatorData::Panic => {
                    let span = self.span_from_bir(basic_block_data.terminator);
                    return Err(error!(span, "panic").eyre(self.interpreter.db()));
                }
            }
        }
    }

    fn evaluate_bir_expr(&mut self, expr: bir::Expr) -> eyre::Result<Value> {
        match expr.data(self.tables) {
            bir::ExprData::BooleanLiteral(value) => Ok(Value::new(self.interpreter, *value)),
            bir::ExprData::IntegerLiteral(value) => Ok(Value::new(self.interpreter, *value)),
            bir::ExprData::StringLiteral(value) => Ok(Value::new(self.interpreter, *value)),
            bir::ExprData::ShareValue(expr) => {
                self.evaluate_bir_expr(*expr)?.into_share(self.interpreter)
            }
            bir::ExprData::Share(place) => self.with_place(*place, Value::share),
            bir::ExprData::Lease(place) => self.with_place(*place, Value::lease),
            bir::ExprData::Give(place) => self.with_place_mut(*place, Value::give),
            bir::ExprData::Tuple(places) => {
                let fields = places
                    .iter()
                    .map(|place| self.give_place(*place))
                    .collect::<eyre::Result<Vec<_>>>()?;
                Ok(Value::new(self.interpreter, Tuple { fields }))
            }
            bir::ExprData::Op(lhs, op, rhs) => {
                let lhs = self.with_place(*lhs, Value::share)?;
                let rhs = self.with_place(*rhs, Value::share)?;
                lhs.read(self.interpreter, |lhs| {
                    rhs.read(self.interpreter, |rhs| self.apply_op(expr, lhs, *op, rhs))
                })
            }
            bir::ExprData::Error => {
                let span = self.span_from_bir(expr);
                Err(error!(span, "compilation error").eyre(self.interpreter.db()))
            }
            bir::ExprData::Unit => Ok(Value::new(self.interpreter, ())),
        }
    }

    fn give_place(&mut self, place: bir::Place) -> eyre::Result<Value> {
        self.with_place_mut(place, Value::give)
    }

    fn tick_clock(&self, expr: impl HasOriginIn<bir::Origins, Origin = syntax::Expr>) {
        self.interpreter.tick_clock(self.span_from_bir(expr));
    }

    fn span_from_bir(
        &self,
        expr: impl HasOriginIn<bir::Origins, Origin = syntax::Expr>,
    ) -> FileSpan {
        self.interpreter.span_from_bir(self.bir, expr)
    }

    fn assign_place(&mut self, place: bir::Place, value: Value) -> eyre::Result<()> {
        match place.data(self.tables) {
            bir::PlaceData::LocalVariable(local_variable) => {
                // FIXME: Presently infallible, but think about atomic etc eventually. =)
                let slot = &mut self.local_variables[*local_variable];
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
                    .eyre(self.interpreter.db()))
            }
            bir::PlaceData::Class(class) => {
                let span_now = self.interpreter.span_now();
                let name = class.name(self.db()).as_str(self.db());
                let name_span = class.name_span(self.db());
                Err(error!(span_now, "cannot assign to `{}`", name)
                    .secondary_label(name_span, &format!("`{}` is a class, declared here", name))
                    .eyre(self.interpreter.db()))
            }
            bir::PlaceData::Intrinsic(intrinsic) => {
                let span_now = self.interpreter.span_now();
                let name = intrinsic.as_str(self.db());
                Err(error!(span_now, "cannot assign to `{}`", name).eyre(self.interpreter.db()))
            }
            bir::PlaceData::Dot(owner_place, field_name) => {
                self.with_place(*owner_place, |owner_value, interpreter| {
                    owner_value.write(interpreter, |data| {
                        data.assign_field(interpreter, *field_name, value)
                    })
                })
            }
        }
    }
    fn with_place<R>(
        &mut self,
        place: bir::Place,
        op: impl FnOnce(&Value, &Interpreter) -> eyre::Result<R>,
    ) -> eyre::Result<R> {
        self.with_place_mut(place, |value, interpreter| op(&*value, interpreter))
    }

    fn with_place_mut<R>(
        &mut self,
        place: bir::Place,
        op: impl FnOnce(&mut Value, &Interpreter) -> eyre::Result<R>,
    ) -> eyre::Result<R> {
        match place.data(self.tables) {
            bir::PlaceData::LocalVariable(local_variable) => {
                op(&mut self.local_variables[*local_variable], self.interpreter)
            }
            bir::PlaceData::Function(function) => op(
                &mut Value::our(self.interpreter, *function),
                self.interpreter,
            ),
            bir::PlaceData::Class(class) => {
                op(&mut Value::our(self.interpreter, *class), self.interpreter)
            }
            bir::PlaceData::Intrinsic(intrinsic) => op(
                &mut Value::our(self.interpreter, *intrinsic),
                self.interpreter,
            ),
            bir::PlaceData::Dot(place, word) => self
                .with_place_mut_box(*place, |value, interpreter| {
                    value.field_mut(interpreter, *word, |v| op(v, interpreter))
                }),
        }
    }

    /// Hack that invokes `with_place` after boxing and using dyn trait;
    /// without this, we get infinite monomorphic expansion for `PlaceData::Dot`.
    fn with_place_mut_box<R>(
        &mut self,
        place: bir::Place,
        op: impl FnOnce(&mut Value, &Interpreter) -> eyre::Result<R>,
    ) -> eyre::Result<R> {
        let op: Box<dyn FnOnce(&mut Value, &Interpreter) -> eyre::Result<R>> = Box::new(op);
        self.with_place_mut(place, op)
    }

    fn eval_place_to_bool(&mut self, place: bir::Place) -> eyre::Result<bool> {
        self.with_place(place, |value, interpreter| {
            value.read(interpreter, |data| data.to_bool(interpreter))
        })
    }

    async fn evaluate_terminator_expr(
        &mut self,
        expr: &bir::TerminatorExpr,
    ) -> eyre::Result<Value> {
        match expr {
            bir::TerminatorExpr::Await(place) => {
                let value = self.give_place(*place)?;
                let data = value.prepare_for_await(self.interpreter)?;
                let thunk = data.into_thunk(self.interpreter)?;
                thunk.invoke(self.interpreter, Some(self)).await
            }
            bir::TerminatorExpr::Call {
                function: function_place,
                arguments: argument_places,
                labels: argument_labels,
            } => {
                let function_value = self.give_place(*function_place)?;
                let argument_values = argument_places
                    .iter()
                    .map(|argument_place| self.give_place(*argument_place))
                    .collect::<eyre::Result<Vec<_>>>()?;
                function_value.read(self.interpreter, |data| {
                    data.call(
                        self.interpreter,
                        argument_values,
                        argument_labels,
                        Some(self),
                    )
                })
            }
        }
    }

    fn apply_op(&self, expr: bir::Expr, lhs: &Data, op: Op, rhs: &Data) -> eyre::Result<Value> {
        let op_error = || {
            let span = self.span_from_bir(expr);
            Err(error!(
                span,
                "cannot apply operator {} to {} and {}",
                op,
                lhs.kind_str(self.interpreter),
                rhs.kind_str(self.interpreter)
            )
            .eyre(self.interpreter.db()))
        };
        let div_zero_error = || {
            let span = self.span_from_bir(expr);
            Err(error!(span, "divide by zero").eyre(self.interpreter.db()))
        };
        let overflow_error = || {
            let span = self.span_from_bir(expr);
            Err(error!(span, "overflow").eyre(self.interpreter.db()))
        };
        match (lhs, rhs) {
            (Data::Bool(lhs), Data::Bool(rhs)) => match op {
                Op::EqualEqual => Ok(Value::new(self.interpreter, lhs == rhs)),
                _ => op_error(),
            },
            (Data::Uint(lhs), Data::Uint(rhs)) => match op {
                Op::EqualEqual => Ok(Value::new(self.interpreter, lhs == rhs)),
                Op::Plus => match lhs.checked_add(*rhs) {
                    Some(value) => Ok(Value::new(self.interpreter, value)),
                    None => overflow_error(),
                },
                Op::Minus => match lhs.checked_sub(*rhs) {
                    Some(value) => Ok(Value::new(self.interpreter, value)),
                    None => overflow_error(),
                },
                Op::Times => match lhs.checked_mul(*rhs) {
                    Some(value) => Ok(Value::new(self.interpreter, value)),
                    None => overflow_error(),
                },
                Op::DividedBy => match lhs.checked_div(*rhs) {
                    Some(value) => Ok(Value::new(self.interpreter, value)),
                    None => div_zero_error(),
                },
                Op::LessThan => Ok(Value::new(self.interpreter, lhs < rhs)),
                Op::GreaterThan => Ok(Value::new(self.interpreter, lhs > rhs)),
            },
            (Data::Int(lhs), Data::Int(rhs)) => match op {
                Op::EqualEqual => Ok(Value::new(self.interpreter, lhs == rhs)),
                Op::Plus => match lhs.checked_add(*rhs) {
                    Some(value) => Ok(Value::new(self.interpreter, value)),
                    None => overflow_error(),
                },
                Op::Minus => match lhs.checked_sub(*rhs) {
                    Some(value) => Ok(Value::new(self.interpreter, value)),
                    None => overflow_error(),
                },
                Op::Times => match lhs.checked_mul(*rhs) {
                    Some(value) => Ok(Value::new(self.interpreter, value)),
                    None => overflow_error(),
                },
                Op::DividedBy => match lhs.checked_div(*rhs) {
                    Some(value) => Ok(Value::new(self.interpreter, value)),
                    None => {
                        if *rhs != -1 {
                            div_zero_error()
                        } else {
                            let span = self.span_from_bir(expr);
                            Err(error!(span, "signed division overflow")
                                .eyre(self.interpreter.db()))
                        }
                    }
                },
                Op::LessThan => Ok(Value::new(self.interpreter, lhs < rhs)),
                Op::GreaterThan => Ok(Value::new(self.interpreter, lhs > rhs)),
            },
            (Data::String(lhs), Data::String(rhs)) => match op {
                Op::EqualEqual => Ok(Value::new(self.interpreter, lhs == rhs)),
                _ => op_error(),
            },
            (Data::Unit(lhs), Data::Unit(rhs)) => match op {
                Op::EqualEqual => Ok(Value::new(self.interpreter, lhs == rhs)),
                _ => op_error(),
            },
            _ => op_error(),
        }
    }
}
