use dada_id::prelude::*;
use dada_ir::{
    class::Class,
    code::{
        bir::{self, TerminatorData, TerminatorExpr},
        syntax,
    },
    error,
    in_ir_db::InIrDbExt,
    origin_table::HasOriginIn,
    span::FileSpan,
    storage::{Atomic, Joint, Leased, SpannedSpecifier, Specifier},
    word::Word,
};
use salsa::DebugWithDb;

use crate::{
    error::DiagnosticBuilderExt,
    heap_graph::HeapGraph,
    kernel::Kernel,
    machine::{
        op::MachineOp, Object, ObjectData, ProgramCounter, Tuple, ValidPermissionData, Value,
    },
    thunk::RustThunk,
};

use self::traversal::PlaceTraversal;

mod access;
mod address;
mod apply_op;
mod apply_unary;
mod assert_invariants;
mod await_thunk;
mod call;
mod concatenate;
mod gc;
mod give;
mod intrinsic;
mod lease;
mod reserve;
mod revoke;
mod share;
mod shlease;
mod tenant;
mod traversal;

pub(crate) struct Stepper<'me> {
    db: &'me dyn crate::Db,
    machine: &'me mut dyn MachineOp,

    /// Kernel for core operations. This is normally `Some`, but we sometimes
    /// temporarily swap with `None` for callbacks.
    kernel: Option<&'me mut dyn Kernel>,
}

impl std::fmt::Debug for Stepper<'_> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_tuple("Stepper").field(&self.machine).finish()
    }
}

/// Specifies the job of the caller after calling `step`, presuming
/// that they wish to continue execution.
pub(crate) enum ControlFlow {
    /// Caller's job is to call `step` again.
    Next,

    /// Execution completed from the given PC with the given value;
    /// caller can inspect the value (`Stepper::check_is_unit` might be useful!).
    Done(ProgramCounter, Value),

    /// Caller's job is to await the thunk by invoking
    /// `RustThunk::invoke`, and then start calling `step` again.
    Await(RustThunk),
}

impl<'me> Stepper<'me> {
    pub(crate) fn new(
        db: &'me dyn crate::Db,
        machine: &'me mut dyn MachineOp,
        kernel: &'me mut dyn Kernel,
    ) -> Self {
        Self {
            db,
            machine,
            kernel: Some(kernel),
        }
    }

    /// Advances execution by a single step, returning either an error
    /// or an indication of what caller should do next.
    ///
    /// Note that this function is synchronous: it never awaits or does I/O.
    #[tracing::instrument(level = "Debug", skip(self))]
    pub(crate) fn step(&mut self) -> eyre::Result<ControlFlow> {
        let mut pc = self.machine.pc();
        let bir_data = pc.bir.data(self.db);
        let table = &bir_data.tables;

        let basic_block_data = pc.basic_block.data(table);

        // The statement should either be the index of a statement or
        // the terminator.
        assert!(
            pc.statement <= basic_block_data.statements.len(),
            "statement index out of range"
        );

        let pc_span = pc.span(self.db);
        let snippet = pc_span.snippet(self.db);
        if snippet.len() > 50 {
            tracing::debug!(
                "executing: {:?}...{:?}",
                &snippet[..25],
                &snippet[snippet.len() - 25..]
            );
        } else {
            tracing::debug!("executing {:?}", snippet);
        }

        if pc.statement < basic_block_data.statements.len() {
            self.step_statement(table, pc.bir, basic_block_data.statements[pc.statement])?;
            pc.statement += 1;
            self.machine.set_pc(pc);
            self.gc(&[]);
            self.assert_invariants()?;
            return Ok(ControlFlow::Next);
        }

        let cf = self.step_terminator(table, pc, basic_block_data.terminator)?;
        let temp;
        self.gc(match &cf {
            ControlFlow::Next => &[],
            ControlFlow::Await(v) => &v.arguments[..],
            ControlFlow::Done(_, v) => {
                temp = [*v];
                &temp
            }
        });
        self.assert_invariants()?;
        Ok(cf)
    }

    /// After a `ControlFlow::Await` is returned, the caller is responsible for
    /// invoking `awaken` with the resulting value. After awaken is called,
    /// the caller should start calling `step` again.
    ///
    /// (This is done by the `RustThunk::invoke` method.)
    pub(crate) fn awaken(&mut self, value: Value) -> eyre::Result<()> {
        self.resume_with(value)
    }

    /// Checks the return value from the `main` function.
    pub(crate) async fn print_if_not_unit(
        &mut self,
        await_pc: ProgramCounter,
        value: Value,
    ) -> eyre::Result<()> {
        match &self.machine[value.object] {
            ObjectData::Unit(()) => Ok(()),
            _ => {
                self.intrinsic_print_async(await_pc, value).await?;
                Ok(())
            }
        }
    }

    fn step_statement(
        &mut self,
        table: &bir::Tables,
        bir: bir::Bir,
        statement: bir::Statement,
    ) -> eyre::Result<()> {
        tracing::debug!(
            "statement = {:?}",
            statement.data(table).debug(&bir.in_ir_db(self.db))
        );

        match statement.data(table) {
            bir::StatementData::AssignExpr(place, expr) => {
                // Subtle: The way this is setup, permissions for the target are not
                // canceled until the write occurs. Consider something like this:
                //
                // ```notrust
                // p = Point(22, 44)
                // q = p.lease
                // p.x = q.y
                // ```
                //
                // This works, but the act of assigning to `p.x` cancels the lease from `q`.
                let value = self.eval_expr(table, *expr)?;
                self.assign_value_to_place(table, *place, value)?;
            }
            bir::StatementData::AssignPlace(target_place, source_place) => {
                self.assign_place_to_place(table, *target_place, *source_place)?;
            }
            bir::StatementData::Clear(lv) => {
                let permission = self.machine.expired_permission(None);
                let object = self.machine.unit_object();
                *self.machine.local_mut(*lv) = Value { object, permission };
            }
            bir::StatementData::BreakpointStart(filename, index) => {
                let kernel = self.kernel.take().unwrap();
                let result = kernel.breakpoint_start(self.db, *filename, *index, &mut || {
                    HeapGraph::new(self.db, self.machine, None)
                });
                self.kernel = Some(kernel);
                result?
            }
            bir::StatementData::BreakpointEnd(filename, index, expr, in_flight_place) => {
                let span = self.span_from_syntax_expr(*expr);
                let kernel = self.kernel.take().unwrap();
                let result = kernel.breakpoint_end(self.db, *filename, *index, span, &mut || {
                    let in_flight_value = try { self.peek_place(table, (*in_flight_place)?)? };
                    HeapGraph::new(self.db, self.machine, in_flight_value)
                });
                self.kernel = Some(kernel);
                result?
            }
        }

        Ok(())
    }

    fn peek_place(&mut self, table: &bir::Tables, place: bir::Place) -> Option<Value> {
        let traversal = self.traverse_to_object(table, place).ok()?;
        Some(Value {
            permission: *traversal.accumulated_permissions.traversed.last().unwrap(),
            object: traversal.object,
        })
    }

    fn assign_place_to_place(
        &mut self,
        table: &bir::Tables,
        target_place: bir::TargetPlace,
        source_place: bir::Place,
    ) -> eyre::Result<()> {
        let target_traversal = self.evaluate_target_place(table, target_place)?;

        assert_ne!(
            target_traversal.accumulated_permissions.atomic,
            Atomic::Yes,
            "atomics not yet implemented"
        );

        let specifier = self.specifier(target_traversal.address);

        let value = self.prepare_value_for_specifier(table, specifier, source_place)?;

        self.assign_value_to_traversal(target_traversal, value)
    }

    fn evaluate_target_place(
        &mut self,
        table: &bir::Tables,
        target_place: bir::TargetPlace,
    ) -> eyre::Result<PlaceTraversal> {
        match &table[target_place] {
            bir::TargetPlaceData::LocalVariable(lv) => {
                Ok(self.traverse_to_local_variable(table, *lv))
            }
            bir::TargetPlaceData::Dot(owner, name) => {
                let owner_traversal = self.traverse_to_object(table, *owner)?;
                let owner_traversal = self.confirm_reservation_if_any(table, owner_traversal)?;
                self.traverse_to_object_field(target_place, owner_traversal, *name)
            }
        }
    }

    #[tracing::instrument(level = "Debug", skip(self, table))]
    fn prepare_value_for_specifier(
        &mut self,
        table: &bir::Tables,
        specifier: Option<impl IntoSpecifierAndSpan>,
        source_place: bir::Place,
    ) -> eyre::Result<Value> {
        let (specifier, specifier_span) = match specifier {
            Some(i) => i.into_specifier_and_span(self.db),
            None => return self.give_place(table, source_place),
        };

        tracing::debug!(?specifier);

        let value = match specifier {
            Specifier::My => self.give_place(table, source_place)?,
            Specifier::Our => self.share_place(table, source_place)?,
            Specifier::Leased => self.lease_place(table, source_place)?,
            Specifier::Shleased => self.shlease_place(table, source_place)?,
            Specifier::Any => self.give_place(table, source_place)?,
        };

        let permission = &self.machine[value.permission];
        let valid = permission
            .valid()
            .expect("value to be stored has expired permision");

        if let (true, Leased::Yes) = (specifier.must_be_owned(), valid.leased) {
            let source_place_span = self.span_from_bir(source_place);
            return Err(error!(source_place_span, "more permissions needed")
                .primary_label(format!(
                    "this value is `{}`, which is leased, not owned",
                    valid.as_str()
                ))
                .secondary_label(
                    specifier_span,
                    format!("`{specifier}` requires owned values"),
                )
                .eyre(self.db));
        }

        if let (true, Joint::Yes) = (specifier.must_be_unique(), valid.joint) {
            let source_place_span = self.span_from_bir(source_place);
            return Err(error!(source_place_span, "more permissions needed")
                .primary_label(format!(
                    "this value is `{}`, which is shared, not unique",
                    valid.as_str()
                ))
                .secondary_label(
                    specifier_span,
                    format!("`{specifier}` requires unique access"),
                )
                .eyre(self.db));
        }

        Ok(value)
    }

    fn assign_value_to_place(
        &mut self,
        table: &bir::Tables,
        target_place: bir::TargetPlace,
        value: Value,
    ) -> eyre::Result<()> {
        assert!(self.machine[value.permission].valid().is_some());

        let target_traversal = self.evaluate_target_place(table, target_place)?;
        self.assign_value_to_traversal(target_traversal, value)
    }

    fn assign_value_to_traversal(
        &mut self,
        target_traversal: PlaceTraversal,
        value: Value,
    ) -> eyre::Result<()> {
        self.write_place(&target_traversal)?;
        self.poke(target_traversal.address, value)?;
        Ok(())
    }

    fn step_terminator(
        &mut self,
        table: &bir::Tables,
        pc: ProgramCounter,
        terminator: bir::Terminator,
    ) -> eyre::Result<ControlFlow> {
        tracing::debug!(
            "terminator = {:?}",
            terminator.data(table).debug(&pc.bir.in_ir_db(self.db))
        );

        let terminator_data: &bir::TerminatorData = &table[terminator];

        match terminator_data {
            // FIXME: implement atomics
            TerminatorData::StartAtomic(b)
            | TerminatorData::EndAtomic(b)
            | TerminatorData::Goto(b) => {
                self.machine.set_pc(pc.move_to_block(*b));
                Ok(ControlFlow::Next)
            }
            TerminatorData::If(place, if_true, if_false) => {
                if self.eval_place_to_bool(table, *place)? {
                    self.machine.set_pc(pc.move_to_block(*if_true));
                } else {
                    self.machine.set_pc(pc.move_to_block(*if_false));
                }
                Ok(ControlFlow::Next)
            }

            TerminatorData::Assign(
                destination,
                TerminatorExpr::Call {
                    function,
                    arguments,
                    labels,
                },
                next_block,
            ) => match self.call(table, terminator, *function, arguments, labels)? {
                call::CallResult::Returned(return_value) => {
                    self.assign_value_to_place(table, *destination, return_value)?;
                    self.machine.set_pc(pc.move_to_block(*next_block));
                    Ok(ControlFlow::Next)
                }
                call::CallResult::PushedNewFrame => Ok(ControlFlow::Next),
            },

            TerminatorData::Assign(
                _destination,
                TerminatorExpr::Await(thunk_place),
                _next_block,
            ) => match self.await_thunk(table, *thunk_place)? {
                await_thunk::AwaitResult::PushedNewFrame => Ok(ControlFlow::Next),
                await_thunk::AwaitResult::RustThunk(rust_thunk) => {
                    Ok(ControlFlow::Await(rust_thunk))
                }
            },

            TerminatorData::Return(place) => {
                let return_value = self.give_place(table, *place)?;

                // Before we pop the frame, clear any permissions
                // and run the GC. Any data that is now dead will
                // thus have the revokation location at the end of the
                // callee, rather than the caller.
                self.machine.clear_frame();
                self.gc(&[return_value]);

                // Pop current frame from the stack.
                self.machine.pop_frame();

                // If that was the top frame, we are done.
                // Otherwise, resume the frame we just uncovered.
                if self.machine.top_frame().is_none() {
                    Ok(ControlFlow::Done(pc, return_value))
                } else {
                    self.resume_with(return_value)?;
                    Ok(ControlFlow::Next)
                }
            }
            TerminatorData::Error => {
                let span = self.span_from_bir(terminator);
                Err(error!(span, "compilation error encountered 😢").eyre(self.db))
            }
            TerminatorData::Panic => {
                let span = self.span_from_bir(terminator);
                Err(error!(span, "panic! omg! 😱").eyre(self.db))
            }
        }
    }

    /// When we call a function or await a thunk, we leave the calling
    /// frame on the stack; when the result comes back, we need to store it
    /// in the expected place and jump to the next basic block. Given the resulting
    /// value `value`, this function examines the top stack frame, stores the
    /// value where it needs to go, and adjusts the PC so that stepping can continue.
    ///
    /// # Panics
    ///
    /// Panics if there is no top frame or it is not awaiting the return of a call
    /// or await.
    fn resume_with(&mut self, value: Value) -> eyre::Result<()> {
        let Some(top) = self.machine.top_frame() else {
            unreachable!("no calling frame")
        };

        // Otherwise, this function was invoked from `top`. We have to store the return
        // value into the location where `top` expects it.
        let top_table = &top.pc.bir.data(self.db).tables;
        let top_basic_block_data = &top_table[top.pc.basic_block];
        assert_eq!(
            top.pc.statement,
            top_basic_block_data.statements.len(),
            "calling frame should be at a terminator"
        );

        let TerminatorData::Assign(top_place, _, top_basic_block) = &top_table[top_basic_block_data.terminator] else {
            unreachable!("calling frame should be at an assign terminator")
        };

        // check that the value which was returned didn't get invalidated
        // by the return itself
        if let Some(expired_at) = self.machine[value.permission].expired() {
            return Err(self.report_traversing_expired_permission(top.pc.span(self.db), expired_at));
        }

        let new_pc = top.pc.move_to_block(*top_basic_block);
        self.assign_value_to_place(top_table, *top_place, value)?;
        self.machine.set_pc(new_pc);
        Ok(())
    }

    /// Returns and activates the `Object` found at `place`; they may have side-effects
    /// like cancelling leases and so forth. Returns Err if `place` is invalid or has insufficient
    /// permissions to read.
    fn read_place(&mut self, table: &bir::Tables, place: bir::Place) -> eyre::Result<Object> {
        let traversal = self.traverse_to_object(table, place)?;
        self.read(&traversal)
    }

    fn eval_place_to_bool(&mut self, table: &bir::Tables, place: bir::Place) -> eyre::Result<bool> {
        let object = self.read_place(table, place)?;
        match &self.machine[object] {
            ObjectData::Bool(b) => Ok(*b),
            data => {
                let span = self.span_from_bir(place);
                Err(Self::unexpected_kind(self.db, span, data, "a boolean"))
            }
        }
    }

    fn eval_expr(&mut self, table: &bir::Tables, expr: bir::Expr) -> eyre::Result<Value> {
        match expr.data(table) {
            bir::ExprData::BooleanLiteral(v) => Ok(Value {
                object: self.machine.new_object(ObjectData::Bool(*v)),
                permission: self.machine.new_permission(ValidPermissionData::our()),
            }),
            bir::ExprData::IntegerLiteral(v) => Ok(Value {
                object: self.machine.new_object(ObjectData::Int(*v)),
                permission: self.machine.new_permission(ValidPermissionData::our()),
            }),
            bir::ExprData::UnsignedIntegerLiteral(v) => Ok(Value {
                object: self.machine.new_object(ObjectData::UnsignedInt(*v)),
                permission: self.machine.new_permission(ValidPermissionData::our()),
            }),
            bir::ExprData::SignedIntegerLiteral(v) => Ok(Value {
                object: self.machine.new_object(ObjectData::SignedInt(*v)),
                permission: self.machine.new_permission(ValidPermissionData::our()),
            }),
            bir::ExprData::FloatLiteral(v) => Ok(Value {
                object: self.machine.new_object(ObjectData::Float(v.0)),
                permission: self.machine.new_permission(ValidPermissionData::our()),
            }),
            bir::ExprData::StringLiteral(v) => Ok(Value {
                object: self
                    .machine
                    .new_object(ObjectData::String(v.as_str(self.db).to_string())),
                permission: self.machine.new_permission(ValidPermissionData::our()),
            }),
            bir::ExprData::Unit => Ok(Value {
                object: self.machine.new_object(ObjectData::Unit(())),
                permission: self.machine.new_permission(ValidPermissionData::our()),
            }),
            bir::ExprData::Reserve(place) => self.reserve_place(table, *place),
            bir::ExprData::Share(place) => self.share_place(table, *place),
            bir::ExprData::Lease(place) => self.lease_place(table, *place),
            bir::ExprData::Shlease(place) => self.shlease_place(table, *place),
            bir::ExprData::Give(place) => self.give_place(table, *place),
            bir::ExprData::Tuple(places) => {
                let fields = places
                    .iter()
                    .map(|place| self.give_place(table, *place))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Value {
                    object: self.machine.new_object(ObjectData::Tuple(Tuple { fields })),
                    permission: self.machine.new_permission(ValidPermissionData::my()),
                })
            }
            bir::ExprData::Concatenate(places) => self.concatenate(table, places),
            bir::ExprData::Op(lhs, op, rhs) => {
                let lhs_traversal = self.traverse_to_object(table, *lhs)?;
                let rhs_traversal = self.traverse_to_object(table, *rhs)?;
                self.apply_op(expr, *op, lhs_traversal.object, rhs_traversal.object)
            }
            bir::ExprData::Unary(op, rhs) => {
                let rhs_traversal = self.traverse_to_object(table, *rhs)?;
                self.apply_unary(expr, *op, rhs_traversal.object)
            }
            bir::ExprData::Error => {
                let span = self.span_from_bir(expr);
                Err(error!(span, "compilation error").eyre(self.db))
            }
        }
    }

    fn unexpected_kind(
        db: &dyn crate::Db,
        span: FileSpan,
        object: &ObjectData,
        what: &str,
    ) -> eyre::Report {
        error!(span, "expected {}, found {}", what, object.kind_str(db)).eyre(db)
    }

    fn no_such_field(db: &dyn crate::Db, span: FileSpan, class: Class, name: Word) -> eyre::Report {
        let class_name = class.name(db).as_str(db);
        let class_span = class.name(db).span(db);
        error!(
            span,
            "the class `{}` has no field named `{}`",
            class_name,
            name.as_str(db)
        )
        .secondary_label(
            class_span,
            &format!("the class `{}` is declared here", class_name),
        )
        .eyre(db)
    }

    fn span_from_bir(
        &self,
        expr: impl HasOriginIn<bir::Origins, Origin = syntax::Expr>,
    ) -> FileSpan {
        let bir = self.machine.pc().bir;
        let origins = bir.origins(self.db);
        let syntax_expr = origins[expr];
        self.span_from_syntax_expr(syntax_expr)
    }

    fn span_from_syntax_expr(&self, syntax_expr: syntax::Expr) -> FileSpan {
        let bir = self.machine.pc().bir;
        bir.span_of(self.db, syntax_expr)
    }
}

trait IntoSpecifierAndSpan: std::fmt::Debug {
    fn into_specifier_and_span(self, db: &dyn crate::Db) -> (Specifier, FileSpan);
}

impl IntoSpecifierAndSpan for (Specifier, FileSpan) {
    fn into_specifier_and_span(self, _db: &dyn crate::Db) -> (Specifier, FileSpan) {
        self
    }
}

impl IntoSpecifierAndSpan for SpannedSpecifier {
    fn into_specifier_and_span(self, db: &dyn crate::Db) -> (Specifier, FileSpan) {
        (self.specifier(db), self.span(db))
    }
}
