use dada_id::prelude::*;
use dada_ir::{
    code::{
        bir::{self, BirData},
        validated::{self, ExprOrigin},
    },
    storage::Atomic,
};
use dada_parse::prelude::*;
use salsa::DebugWithDb;

use crate::{
    brewery::{Brewery, LoopContext},
    cursor::Cursor,
};

/// Convert a [`validated::Tree`] to [BIR](`bir::Bir`).
#[salsa::tracked]
pub fn brew(db: &dyn crate::Db, validated_tree: validated::Tree) -> bir::Bir {
    let function = validated_tree.function(db);
    let breakpoints = dada_breakpoint::locations::breakpoints_in_tree(
        db,
        function.input_file(db),
        function.syntax_tree(db),
    );
    let mut tables = bir::Tables::default();
    let mut origins = bir::Origins::default();
    let brewery = &mut Brewery::new(
        db,
        function.input_file(db),
        &breakpoints,
        validated_tree,
        &mut tables,
        &mut origins,
    );
    let num_parameters = validated_tree.data(db).num_parameters;

    // Compile the root expression and -- assuming it doesn't diverge --
    // return the resulting value.
    let root_expr = validated_tree.data(db).root_expr;
    let root_expr_origin = validated_tree.origins(db)[root_expr];
    let mut cursor = Cursor::new(brewery, root_expr_origin);
    if let Some(place) = cursor.brew_expr_to_temporary(brewery, root_expr) {
        cursor.terminate_and_diverge(
            brewery,
            bir::TerminatorData::Return(place),
            root_expr_origin,
        );
    }
    let start_basic_block = cursor.complete();

    let bir = bir::Bir::new(
        db,
        function.input_file(db),
        function.name(db),
        function.syntax_tree(db),
        BirData::new(tables, num_parameters, start_basic_block),
        origins,
    );

    tracing::trace!(
        "brew(function={:?}) = {:#?}",
        function.debug(db),
        bir.debug(db)
    );

    bir
}

impl Cursor {
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn brew_expr_for_side_effects(
        &mut self,
        brewery: &mut Brewery<'_>,
        expr: validated::Expr,
    ) {
        tracing::debug!("expr = {:?}", expr.data(brewery.validated_tables()));
        let origin = brewery.origin(expr);
        let temporary_scope = self.push_temporary_scope(brewery);
        match expr.data(brewery.validated_tables()) {
            validated::ExprData::Break {
                from_expr,
                with_value,
            } => {
                self.push_breakpoint_start(brewery, origin);
                let loop_context = brewery.loop_context(*from_expr);
                self.brew_expr_and_assign_to(brewery, loop_context.loop_value, *with_value);
                self.push_breakpoint_end(brewery, Some(loop_context.loop_value), origin);
                self.terminate_and_goto(brewery, loop_context.break_block, origin);
            }

            validated::ExprData::Continue(from_expr) => {
                self.push_breakpoint_start(brewery, origin);
                let loop_context = brewery.loop_context(*from_expr);
                self.push_breakpoint_end(brewery, None::<bir::Place>, origin);
                self.terminate_and_goto(brewery, loop_context.continue_block, origin);
            }

            validated::ExprData::Return(value_expr) => {
                self.push_breakpoint_start(brewery, origin);
                if let Some(value_place) = self.brew_expr_to_temporary(brewery, *value_expr) {
                    self.push_breakpoint_end(brewery, Some(value_place), origin);
                    self.terminate_and_diverge(
                        brewery,
                        bir::TerminatorData::Return(value_place),
                        origin,
                    );
                }
            }

            validated::ExprData::Error => {
                self.push_breakpoint_start(brewery, origin);
                self.push_breakpoint_end(brewery, None::<bir::Place>, origin);
                self.terminate_and_diverge(brewery, bir::TerminatorData::Error, origin)
            }

            validated::ExprData::Assign(target_place, value_expr) => {
                let (target_place, target_origins) = self.brew_target_place(brewery, *target_place);

                self.push_breakpoint_starts(brewery, target_origins.iter().copied(), origin);
                self.brew_expr_and_assign_to(brewery, target_place, *value_expr);
                self.push_breakpoint_ends(brewery, None::<bir::Place>, target_origins, origin)
            }

            validated::ExprData::Declare(vars, subexpr) => {
                self.push_breakpoint_start(brewery, origin);
                self.brew_expr_for_side_effects(brewery, *subexpr);
                self.pop_declared_variables(brewery, vars, origin);
                self.push_breakpoint_end(brewery, None::<bir::Place>, origin);
            }

            validated::ExprData::Await(_)
            | validated::ExprData::If(_, _, _)
            | validated::ExprData::Loop(_)
            | validated::ExprData::Seq(_)
            | validated::ExprData::Op(_, _, _)
            | validated::ExprData::Unary(_, _)
            | validated::ExprData::BooleanLiteral(_)
            | validated::ExprData::IntegerLiteral(_)
            | validated::ExprData::UnsignedIntegerLiteral(_)
            | validated::ExprData::SignedIntegerLiteral(_)
            | validated::ExprData::FloatLiteral(_)
            | validated::ExprData::StringLiteral(_)
            | validated::ExprData::Call(_, _)
            | validated::ExprData::IntoShared(_)
            | validated::ExprData::Lease(_)
            | validated::ExprData::Share(_)
            | validated::ExprData::Give(_)
            | validated::ExprData::Tuple(_)
            | validated::ExprData::Concatenate(_)
            | validated::ExprData::Atomic(_) => {
                let _ = self.brew_expr_to_temporary(brewery, expr);
            }
        }
        self.pop_temporary_scope(brewery, temporary_scope);
    }

    /// Compiles expr into a temporary `t` and returns `Some(t)`.
    /// Returns `None` if this is dead code.
    pub(crate) fn brew_expr_to_temporary(
        &mut self,
        brewery: &mut Brewery<'_>,
        expr: validated::Expr,
    ) -> Option<bir::Place> {
        let origin = brewery.origin(expr);
        // Spill into a temporary
        let temp_place = add_temporary_place(brewery, origin);
        self.brew_expr_and_assign_to(brewery, temp_place, expr);
        Some(brewery.place_from_target_place(temp_place))
    }

    /// Compiles an expression down to the value it produces.
    ///
    /// Returns `None` if this is an expression (like `break`) that
    /// produces diverging control flow (and hence no value).
    pub(crate) fn brew_expr_and_assign_to(
        &mut self,
        brewery: &mut Brewery<'_>,
        target: bir::TargetPlace,
        expr: validated::Expr,
    ) {
        let origin = brewery.origin(expr);
        let temporary_scope = self.push_temporary_scope(brewery);
        match expr.data(brewery.validated_tables()) {
            validated::ExprData::Await(future) => {
                self.push_breakpoint_start(brewery, origin);
                if let Some(place) = self.brew_expr_to_temporary(brewery, *future) {
                    self.terminate_and_continue(
                        brewery,
                        |next_block| {
                            bir::TerminatorData::Assign(
                                target,
                                bir::TerminatorExpr::Await(place),
                                next_block,
                            )
                        },
                        origin,
                    );
                    self.push_breakpoint_end(brewery, Some(target), origin);
                }
            }

            validated::ExprData::If(condition, if_true, if_false) => {
                self.push_breakpoint_start(brewery, origin);
                if let Some(condition_place) = self.brew_expr_to_temporary(brewery, *condition) {
                    let if_true_block = brewery.dummy_block(origin);
                    let if_false_block = brewery.dummy_block(origin);
                    let join_block = self.terminate_and_continue(
                        brewery,
                        |_| bir::TerminatorData::If(condition_place, if_true_block, if_false_block),
                        origin,
                    );
                    self.push_breakpoint_end(brewery, Some(target), origin); // "cusp" of an if is after it completes

                    let mut if_true_cursor = self.with_end_block(if_true_block);
                    if_true_cursor.brew_expr_and_assign_to(brewery, target, *if_true);
                    if_true_cursor.terminate_and_goto(brewery, join_block, origin);

                    let mut if_false_cursor = self.with_end_block(if_false_block);
                    if_false_cursor.brew_expr_and_assign_to(brewery, target, *if_false);
                    if_false_cursor.terminate_and_goto(brewery, join_block, origin);
                }
            }

            validated::ExprData::Loop(body) => {
                self.push_breakpoint_start(brewery, origin);
                let body_block = brewery.dummy_block(origin);
                let break_block = self.terminate_and_continue(
                    brewery,
                    |_| bir::TerminatorData::Goto(body_block),
                    origin,
                );
                self.push_breakpoint_end(brewery, Some(target), origin); // "cusp" of a loop is after it breaks

                let body_brewery = &mut brewery.subbrewery();
                body_brewery.push_loop_context(
                    expr,
                    LoopContext {
                        continue_block: body_block,
                        break_block,
                        loop_value: target,
                    },
                );
                let mut body_cursor = self.with_end_block(body_block);
                body_cursor.brew_expr_for_side_effects(body_brewery, *body);
                body_cursor.terminate_and_diverge(
                    body_brewery,
                    bir::TerminatorData::Goto(body_block),
                    origin,
                );
            }

            validated::ExprData::IntoShared(operand) => {
                if let Some(temp) = self.brew_expr_to_temporary(brewery, *operand) {
                    self.push_breakpoint_start(brewery, origin);
                    self.push_assignment(brewery, target, bir::ExprData::IntoShared(temp), origin);
                    self.push_breakpoint_end(brewery, Some(target), origin);
                }
            }

            validated::ExprData::Lease(place) => {
                let (place, origins) = self.brew_place(brewery, *place);
                self.push_breakpoint_starts(brewery, origins.iter().copied(), origin);
                self.push_assignment(brewery, target, bir::ExprData::Lease(place), origin);
                self.push_breakpoint_ends(brewery, Some(target), origins, origin);
            }

            validated::ExprData::Share(place) => {
                let (place, origins) = self.brew_place(brewery, *place);
                self.push_breakpoint_starts(brewery, origins.iter().copied(), origin);
                self.push_assignment(brewery, target, bir::ExprData::Share(place), origin);
                self.push_breakpoint_ends(brewery, Some(target), origins, origin);
            }

            validated::ExprData::Give(place) => {
                let (place, origins) = self.brew_place(brewery, *place);
                self.push_breakpoint_starts(brewery, origins.iter().copied(), origin);
                self.push_assignment(brewery, target, bir::ExprData::Give(place), origin);
                self.push_breakpoint_ends(brewery, Some(target), origins, origin)
            }

            validated::ExprData::BooleanLiteral(value) => {
                self.push_breakpoint_start(brewery, origin);
                self.push_assignment(
                    brewery,
                    target,
                    bir::ExprData::BooleanLiteral(*value),
                    origin,
                );
                self.push_breakpoint_end(brewery, Some(target), origin);
            }

            validated::ExprData::SignedIntegerLiteral(value) => {
                self.push_breakpoint_start(brewery, origin);
                self.push_assignment(
                    brewery,
                    target,
                    bir::ExprData::SignedIntegerLiteral(*value),
                    origin,
                );
                self.push_breakpoint_end(brewery, Some(target), origin);
            }

            validated::ExprData::UnsignedIntegerLiteral(value) => {
                self.push_breakpoint_start(brewery, origin);
                self.push_assignment(
                    brewery,
                    target,
                    bir::ExprData::UnsignedIntegerLiteral(*value),
                    origin,
                );
                self.push_breakpoint_end(brewery, Some(target), origin);
            }

            validated::ExprData::IntegerLiteral(value) => {
                self.push_breakpoint_start(brewery, origin);
                self.push_assignment(
                    brewery,
                    target,
                    bir::ExprData::IntegerLiteral(*value),
                    origin,
                );
                self.push_breakpoint_end(brewery, Some(target), origin);
            }

            validated::ExprData::FloatLiteral(value) => {
                self.push_breakpoint_start(brewery, origin);
                self.push_assignment(brewery, target, bir::ExprData::FloatLiteral(*value), origin);
                self.push_breakpoint_end(brewery, Some(target), origin);
            }

            validated::ExprData::StringLiteral(value) => {
                self.push_breakpoint_start(brewery, origin);
                self.push_assignment(
                    brewery,
                    target,
                    bir::ExprData::StringLiteral(*value),
                    origin,
                );
                self.push_breakpoint_end(brewery, Some(target), origin);
            }

            validated::ExprData::Concatenate(exprs) => {
                self.push_breakpoint_start(brewery, origin);
                if let Some(values) = exprs
                    .iter()
                    .map(|expr| self.brew_expr_to_temporary(brewery, *expr))
                    .collect::<Option<Vec<_>>>()
                {
                    assert_eq!(values.len(), exprs.len());
                    self.push_assignment(
                        brewery,
                        target,
                        bir::ExprData::Concatenate(values),
                        origin,
                    );
                    self.push_breakpoint_end(brewery, Some(target), origin);
                }
            }
            validated::ExprData::Tuple(exprs) => {
                self.push_breakpoint_start(brewery, origin);
                if let Some(values) = exprs
                    .iter()
                    .map(|expr| self.brew_expr_to_temporary(brewery, *expr))
                    .collect::<Option<Vec<_>>>()
                {
                    assert_eq!(values.len(), exprs.len());
                    if values.is_empty() {
                        self.push_assignment(brewery, target, bir::ExprData::Unit, origin);
                    } else {
                        assert_ne!(values.len(), 1);
                        self.push_assignment(brewery, target, bir::ExprData::Tuple(values), origin);
                    }
                    self.push_breakpoint_end(brewery, Some(target), origin);
                }
            }

            validated::ExprData::Op(lhs, op, rhs) => {
                self.push_breakpoint_start(brewery, origin);
                if let Some(lhs) = self.brew_expr_to_temporary(brewery, *lhs) {
                    if let Some(rhs) = self.brew_expr_to_temporary(brewery, *rhs) {
                        self.push_assignment(
                            brewery,
                            target,
                            bir::ExprData::Op(lhs, *op, rhs),
                            origin,
                        );
                        self.push_breakpoint_end(brewery, Some(target), origin);
                    }
                }
            }

            validated::ExprData::Unary(op, rhs) => {
                self.push_breakpoint_start(brewery, origin);
                if let Some(rhs) = self.brew_expr_to_temporary(brewery, *rhs) {
                    self.push_assignment(brewery, target, bir::ExprData::Unary(*op, rhs), origin);
                    self.push_breakpoint_end(brewery, Some(target), origin);
                }
            }

            validated::ExprData::Seq(exprs) => {
                self.push_breakpoint_start(brewery, origin);
                if let Some((last_expr, prefix)) = exprs.split_last() {
                    for e in prefix {
                        self.brew_expr_for_side_effects(brewery, *e);
                    }
                    self.brew_expr_and_assign_to(brewery, target, *last_expr);
                    self.push_breakpoint_end(brewery, Some(target), origin);
                } else {
                    self.push_assignment(brewery, target, bir::ExprData::Unit, origin);
                    self.push_breakpoint_end(brewery, Some(target), origin);
                }
            }

            validated::ExprData::Assign(..) => {
                self.brew_expr_for_side_effects(brewery, expr);
                self.push_assignment(brewery, target, bir::ExprData::Unit, origin);
            }

            validated::ExprData::Call(func, args) => {
                self.push_breakpoint_start(brewery, origin);
                if let Some(func_place) = self.brew_expr_to_temporary(brewery, *func) {
                    let mut places = vec![];
                    let mut names = vec![];
                    for arg in args {
                        if let Some((place, name)) = self.brew_named_expr(brewery, *arg) {
                            places.push(place);
                            names.push(name);
                        }
                    }
                    if places.len() == args.len() {
                        self.terminate_and_continue(
                            brewery,
                            |next_block| {
                                bir::TerminatorData::Assign(
                                    target,
                                    bir::TerminatorExpr::Call {
                                        function: func_place,
                                        arguments: places,
                                        labels: names,
                                    },
                                    next_block,
                                )
                            },
                            origin,
                        );
                        self.push_breakpoint_end(brewery, Some(target), origin);
                    }
                }
            }

            validated::ExprData::Atomic(subexpr) => {
                self.push_breakpoint_start(brewery, origin);
                self.terminate_and_continue(brewery, bir::TerminatorData::StartAtomic, origin);

                // FIXME what if a break exits through an atomic?

                self.brew_expr_and_assign_to(brewery, target, *subexpr);

                self.terminate_and_continue(brewery, bir::TerminatorData::EndAtomic, origin);
                self.push_breakpoint_end(brewery, Some(target), origin);
            }

            validated::ExprData::Declare(vars, subexpr) => {
                self.push_breakpoint_start(brewery, origin);
                self.brew_expr_and_assign_to(brewery, target, *subexpr);
                self.pop_declared_variables(brewery, vars, origin);
                self.push_breakpoint_end(brewery, None::<bir::Place>, origin);
            }

            validated::ExprData::Error
            | validated::ExprData::Return(_)
            | validated::ExprData::Continue(_)
            | validated::ExprData::Break { .. } => {
                self.brew_expr_for_side_effects(brewery, expr);
            }
        };
        self.pop_temporary_scope(brewery, temporary_scope);
    }

    /// Brews a place to a bir place, returning a vector of the
    /// syntactical expressions that were evaluated along the way.
    /// No cusp expressions are emitted, as places are evaluated
    /// instantaneously and do not represent a moment in time.
    ///
    /// Example: if you have `a.b.c`, we will evaluate this to the
    /// place `a.b.c` and return a vector with `[a, a.b, a.b.c]`.
    ///
    /// This vector is used to add cusp operations (they all show the value
    /// of `a.b.c` in its entirety, though).
    #[allow(clippy::only_used_in_recursion)]
    pub(crate) fn brew_place(
        &mut self,
        brewery: &mut Brewery<'_>,
        place: validated::Place,
    ) -> (bir::Place, Vec<ExprOrigin>) {
        let origin = brewery.origin(place);
        match place.data(brewery.validated_tables()) {
            validated::PlaceData::LocalVariable(validated_var) => {
                let bir_var = brewery.variable(*validated_var);
                let place = brewery.add(bir::PlaceData::LocalVariable(bir_var), origin);
                (place, vec![origin])
            }
            validated::PlaceData::Function(function) => {
                let place = brewery.add(bir::PlaceData::Function(*function), origin);
                (place, vec![origin])
            }
            validated::PlaceData::Intrinsic(intrinsic) => {
                let place = brewery.add(bir::PlaceData::Intrinsic(*intrinsic), origin);
                (place, vec![origin])
            }
            validated::PlaceData::Class(class) => {
                let place = brewery.add(bir::PlaceData::Class(*class), origin);
                (place, vec![origin])
            }
            validated::PlaceData::Dot(base, field) => {
                let (base, mut origins) = self.brew_place(brewery, *base);
                let place = brewery.add(bir::PlaceData::Dot(base, *field), origin);
                origins.push(origin);
                (place, origins)
            }
        }
    }

    pub(crate) fn brew_target_place(
        &mut self,
        brewery: &mut Brewery<'_>,
        place: validated::TargetPlace,
    ) -> (bir::TargetPlace, Vec<ExprOrigin>) {
        let origin = brewery.origin(place);
        match brewery.validated_tables()[place] {
            validated::TargetPlaceData::LocalVariable(validated_var) => {
                let place = self.brew_target_variable(brewery, validated_var, origin);
                (place, vec![origin])
            }
            validated::TargetPlaceData::Dot(base, field) => {
                let (base, mut origins) = self.brew_place(brewery, base);
                let place = brewery.add(bir::TargetPlaceData::Dot(base, field), origin);
                origins.push(origin);
                (place, origins)
            }
        }
    }
    pub(crate) fn brew_target_variable(
        &mut self,
        brewery: &mut Brewery<'_>,
        validated_var: validated::LocalVariable,
        origin: ExprOrigin,
    ) -> bir::TargetPlace {
        let bir_var = brewery.variable(validated_var);
        brewery.add(bir::TargetPlaceData::LocalVariable(bir_var), origin)
    }
}

fn add_temporary(brewery: &mut Brewery, origin: ExprOrigin) -> bir::LocalVariable {
    let temporary = brewery.add(
        bir::LocalVariableData {
            name: None,
            atomic: Atomic::No,
        },
        validated::LocalVariableOrigin::Temporary(origin.into()),
    );
    tracing::debug!("created temporary: temp{{{:?}}}", u32::from(temporary));
    brewery.push_temporary(temporary);
    temporary
}

fn add_temporary_place(brewery: &mut Brewery, origin: ExprOrigin) -> bir::TargetPlace {
    let temporary_var = add_temporary(brewery, origin);
    brewery.add(bir::TargetPlaceData::LocalVariable(temporary_var), origin)
}
