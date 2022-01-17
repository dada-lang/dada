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
            bir,
            tables: &bir_data.tables,
            local_variables,
            parent_stack_frame,
        };
        Box::pin(stack_frame.execute(self, bir_data.start_basic_block))
    }
}

impl StackFrame<'_> {
    async fn execute(
        mut self,
        interpreter: &Interpreter<'_>,
        mut basic_block: bir::BasicBlock,
    ) -> eyre::Result<Value> {
        loop {
            let basic_block_data = basic_block.data(self.tables);
            for statement in &basic_block_data.statements {
                self.tick_clock(interpreter, *statement);
                match statement.data(self.tables) {
                    dada_ir::code::bir::StatementData::Assign(place, expr) => {
                        let expr_value = self.evaluate_bir_expr(interpreter, *expr)?;
                        self.assign_place(interpreter, *place, expr_value)?;
                    }
                }
            }

            self.tick_clock(interpreter, basic_block_data.terminator);
            match basic_block_data.terminator.data(self.tables) {
                dada_ir::code::bir::TerminatorData::Goto(next_block) => {
                    basic_block = *next_block;
                }
                dada_ir::code::bir::TerminatorData::If(place, if_true, if_false) => {
                    if self.eval_place_to_bool(interpreter, *place)? {
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
                    return self.give_place(interpreter, *place);
                }
                dada_ir::code::bir::TerminatorData::Assign(place, expr, next) => {
                    let value = self.evaluate_terminator_expr(interpreter, expr).await?;
                    self.assign_place(interpreter, *place, value)?;
                    basic_block = *next;
                }
                dada_ir::code::bir::TerminatorData::Error => {
                    let span = self.span_from_bir(interpreter, basic_block_data.terminator);
                    return Err(error!(span, "compilation error").eyre(interpreter.db()));
                }
                dada_ir::code::bir::TerminatorData::Panic => {
                    let span = self.span_from_bir(interpreter, basic_block_data.terminator);
                    return Err(error!(span, "panic").eyre(interpreter.db()));
                }
            }
        }
    }

    fn evaluate_bir_expr(
        &mut self,
        interpreter: &Interpreter<'_>,
        expr: bir::Expr,
    ) -> eyre::Result<Value> {
        match expr.data(self.tables) {
            bir::ExprData::BooleanLiteral(value) => Ok(Value::new(interpreter, *value)),
            bir::ExprData::IntegerLiteral(value) => Ok(Value::new(interpreter, *value)),
            bir::ExprData::StringLiteral(value) => Ok(Value::new(interpreter, *value)),
            bir::ExprData::ShareValue(expr) => self
                .evaluate_bir_expr(interpreter, *expr)?
                .into_share(interpreter),
            bir::ExprData::Share(place) => self.with_place(interpreter, *place, Value::share),
            bir::ExprData::Lease(place) => self.with_place(interpreter, *place, Value::lease),
            bir::ExprData::Give(place) => self.with_place_mut(interpreter, *place, Value::give),
            bir::ExprData::Tuple(places) => {
                let fields = places
                    .iter()
                    .map(|place| self.give_place(interpreter, *place))
                    .collect::<eyre::Result<Vec<_>>>()?;
                Ok(Value::new(interpreter, Tuple { fields }))
            }
            bir::ExprData::Op(lhs, op, rhs) => {
                let lhs = self.with_place(interpreter, *lhs, Value::share)?;
                let rhs = self.with_place(interpreter, *rhs, Value::share)?;
                lhs.read(interpreter, |lhs| {
                    rhs.read(interpreter, |rhs| {
                        self.apply_op(interpreter, expr, lhs, *op, rhs)
                    })
                })
            }
            bir::ExprData::Error => {
                let span = self.span_from_bir(interpreter, expr);
                Err(error!(span, "compilation error").eyre(interpreter.db()))
            }
            bir::ExprData::Unit => Ok(Value::new(interpreter, ())),
        }
    }

    fn give_place(
        &mut self,
        interpreter: &Interpreter<'_>,
        place: bir::Place,
    ) -> eyre::Result<Value> {
        self.with_place_mut(interpreter, place, Value::give)
    }

    fn tick_clock(
        &self,
        interpreter: &Interpreter<'_>,
        expr: impl HasOriginIn<bir::Origins, Origin = syntax::Expr>,
    ) {
        interpreter.tick_clock(self.span_from_bir(interpreter, expr));
    }

    fn span_from_bir(
        &self,
        interpreter: &Interpreter<'_>,
        expr: impl HasOriginIn<bir::Origins, Origin = syntax::Expr>,
    ) -> FileSpan {
        interpreter.span_from_bir(self.bir, expr)
    }

    fn assign_place(
        &mut self,
        interpreter: &Interpreter<'_>,
        place: bir::Place,
        value: Value,
    ) -> eyre::Result<()> {
        match place.data(self.tables) {
            bir::PlaceData::LocalVariable(local_variable) => {
                // FIXME: Presently infallible, but think about atomic etc eventually. =)
                let slot = &mut self.local_variables[*local_variable];
                *slot = value;
                Ok(())
            }
            bir::PlaceData::Function(function) => {
                let span_now = interpreter.span_now();
                let name = function.name(interpreter.db()).as_str(interpreter.db());
                let name_span = function.name_span(interpreter.db());
                Err(error!(span_now, "cannot assign to `{}`", name)
                    .secondary_label(
                        name_span,
                        &format!("`{}` is a function, declared here", name),
                    )
                    .eyre(interpreter.db()))
            }
            bir::PlaceData::Class(class) => {
                let span_now = interpreter.span_now();
                let name = class.name(interpreter.db()).as_str(interpreter.db());
                let name_span = class.name_span(interpreter.db());
                Err(error!(span_now, "cannot assign to `{}`", name)
                    .secondary_label(name_span, &format!("`{}` is a class, declared here", name))
                    .eyre(interpreter.db()))
            }
            bir::PlaceData::Intrinsic(intrinsic) => {
                let span_now = interpreter.span_now();
                let name = intrinsic.as_str(interpreter.db());
                Err(error!(span_now, "cannot assign to `{}`", name).eyre(interpreter.db()))
            }
            bir::PlaceData::Dot(owner_place, field_name) => {
                self.with_place(interpreter, *owner_place, |owner_value, interpreter| {
                    owner_value.write(interpreter, |data| {
                        data.assign_field(interpreter, *field_name, value)
                    })
                })
            }
        }
    }

    fn with_place<R>(
        &mut self,
        interpreter: &Interpreter<'_>,
        place: bir::Place,
        op: impl FnOnce(&Value, &Interpreter) -> eyre::Result<R>,
    ) -> eyre::Result<R> {
        self.with_place_mut(interpreter, place, |value, interpreter| {
            op(&*value, interpreter)
        })
    }

    fn with_place_mut<R>(
        &mut self,
        interpreter: &Interpreter<'_>,
        place: bir::Place,
        op: impl FnOnce(&mut Value, &Interpreter) -> eyre::Result<R>,
    ) -> eyre::Result<R> {
        match place.data(self.tables) {
            bir::PlaceData::LocalVariable(local_variable) => {
                op(&mut self.local_variables[*local_variable], interpreter)
            }
            bir::PlaceData::Function(function) => {
                op(&mut Value::our(interpreter, *function), interpreter)
            }
            bir::PlaceData::Class(class) => op(&mut Value::our(interpreter, *class), interpreter),
            bir::PlaceData::Intrinsic(intrinsic) => {
                op(&mut Value::our(interpreter, *intrinsic), interpreter)
            }
            bir::PlaceData::Dot(place, word) => {
                self.with_place_mut_box(interpreter, *place, |value, interpreter| {
                    value.field_mut(interpreter, *word, |v| op(v, interpreter))
                })
            }
        }
    }

    /// Hack that invokes `with_place` after boxing and using dyn trait;
    /// without this, we get infinite monomorphic expansion for `PlaceData::Dot`.
    fn with_place_mut_box<R>(
        &mut self,
        interpreter: &Interpreter<'_>,
        place: bir::Place,
        op: impl FnOnce(&mut Value, &Interpreter) -> eyre::Result<R>,
    ) -> eyre::Result<R> {
        let op: Box<dyn FnOnce(&mut Value, &Interpreter) -> eyre::Result<R>> = Box::new(op);
        self.with_place_mut(interpreter, place, op)
    }

    fn eval_place_to_bool(
        &mut self,
        interpreter: &Interpreter<'_>,
        place: bir::Place,
    ) -> eyre::Result<bool> {
        self.with_place(interpreter, place, |value, interpreter| {
            value.read(interpreter, |data| data.to_bool(interpreter))
        })
    }

    async fn evaluate_terminator_expr(
        &mut self,
        interpreter: &Interpreter<'_>,
        expr: &bir::TerminatorExpr,
    ) -> eyre::Result<Value> {
        match expr {
            bir::TerminatorExpr::Await(place) => {
                let value = self.give_place(interpreter, *place)?;
                let data = value.prepare_for_await(interpreter)?;
                let thunk = data.into_thunk(interpreter)?;
                thunk.invoke(interpreter, Some(self)).await
            }
            bir::TerminatorExpr::Call {
                function: function_place,
                arguments: argument_places,
                labels: argument_labels,
            } => {
                let function_value = self.give_place(interpreter, *function_place)?;
                let argument_values = argument_places
                    .iter()
                    .map(|argument_place| self.give_place(interpreter, *argument_place))
                    .collect::<eyre::Result<Vec<_>>>()?;
                function_value.read(interpreter, |data| {
                    data.call(interpreter, argument_values, argument_labels, Some(self))
                })
            }
        }
    }

    fn apply_op(
        &self,
        interpreter: &Interpreter<'_>,
        expr: bir::Expr,
        lhs: &Data,
        op: Op,
        rhs: &Data,
    ) -> eyre::Result<Value> {
        let op_error = || {
            let span = self.span_from_bir(interpreter, expr);
            Err(error!(
                span,
                "cannot apply operator {} to {} and {}",
                op,
                lhs.kind_str(interpreter),
                rhs.kind_str(interpreter)
            )
            .eyre(interpreter.db()))
        };
        let div_zero_error = || {
            let span = self.span_from_bir(interpreter, expr);
            Err(error!(span, "divide by zero").eyre(interpreter.db()))
        };
        let overflow_error = || {
            let span = self.span_from_bir(interpreter, expr);
            Err(error!(span, "overflow").eyre(interpreter.db()))
        };
        match (lhs, rhs) {
            (Data::Bool(lhs), Data::Bool(rhs)) => match op {
                Op::EqualEqual => Ok(Value::new(interpreter, lhs == rhs)),
                _ => op_error(),
            },
            (Data::Uint(lhs), Data::Uint(rhs)) => match op {
                Op::EqualEqual => Ok(Value::new(interpreter, lhs == rhs)),
                Op::Plus => match lhs.checked_add(*rhs) {
                    Some(value) => Ok(Value::new(interpreter, value)),
                    None => overflow_error(),
                },
                Op::Minus => match lhs.checked_sub(*rhs) {
                    Some(value) => Ok(Value::new(interpreter, value)),
                    None => overflow_error(),
                },
                Op::Times => match lhs.checked_mul(*rhs) {
                    Some(value) => Ok(Value::new(interpreter, value)),
                    None => overflow_error(),
                },
                Op::DividedBy => match lhs.checked_div(*rhs) {
                    Some(value) => Ok(Value::new(interpreter, value)),
                    None => div_zero_error(),
                },
                Op::LessThan => Ok(Value::new(interpreter, lhs < rhs)),
                Op::GreaterThan => Ok(Value::new(interpreter, lhs > rhs)),
            },
            (Data::Int(lhs), Data::Int(rhs)) => match op {
                Op::EqualEqual => Ok(Value::new(interpreter, lhs == rhs)),
                Op::Plus => match lhs.checked_add(*rhs) {
                    Some(value) => Ok(Value::new(interpreter, value)),
                    None => overflow_error(),
                },
                Op::Minus => match lhs.checked_sub(*rhs) {
                    Some(value) => Ok(Value::new(interpreter, value)),
                    None => overflow_error(),
                },
                Op::Times => match lhs.checked_mul(*rhs) {
                    Some(value) => Ok(Value::new(interpreter, value)),
                    None => overflow_error(),
                },
                Op::DividedBy => match lhs.checked_div(*rhs) {
                    Some(value) => Ok(Value::new(interpreter, value)),
                    None => {
                        if *rhs != -1 {
                            div_zero_error()
                        } else {
                            let span = self.span_from_bir(interpreter, expr);
                            Err(error!(span, "signed division overflow").eyre(interpreter.db()))
                        }
                    }
                },
                Op::LessThan => Ok(Value::new(interpreter, lhs < rhs)),
                Op::GreaterThan => Ok(Value::new(interpreter, lhs > rhs)),
            },
            (Data::String(lhs), Data::String(rhs)) => match op {
                Op::EqualEqual => Ok(Value::new(interpreter, lhs == rhs)),
                _ => op_error(),
            },
            (Data::Unit(lhs), Data::Unit(rhs)) => match op {
                Op::EqualEqual => Ok(Value::new(interpreter, lhs == rhs)),
                _ => op_error(),
            },
            _ => op_error(),
        }
    }
}
