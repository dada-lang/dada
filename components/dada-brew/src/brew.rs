use dada_id::prelude::*;
use dada_ir::{
    code::{
        bir::{self, BirData},
        syntax, validated,
    },
    origin_table::HasOriginIn,
    storage_mode::StorageMode,
};

use crate::{
    brewery::{Brewery, LoopContext},
    cursor::Cursor,
};

#[salsa::memoized(in crate::Jar)]
pub fn brew(db: &dyn crate::Db, validated_tree: validated::Tree) -> bir::Bir {
    let origin = validated_tree.origin(db);
    let mut tables = bir::Tables::default();
    let mut origins = bir::Origins::default();
    let brewery = &mut Brewery::new(db, validated_tree, &mut tables, &mut origins);

    // Compile the root expression and -- assuming it doesn't diverse --
    // return the resulting value.
    let root_expr = validated_tree.data(db).root_expr;
    let root_expr_origin = *root_expr.origin_in(validated_tree.origins(db));
    let mut cursor = Cursor::new(brewery, root_expr_origin);
    if let Some(place) = cursor.brew_expr_to_place(brewery, root_expr) {
        cursor.terminate_and_diverge(
            brewery,
            bir::TerminatorData::Return(place),
            root_expr_origin,
        );
    }
    let start_basic_block = cursor.complete();

    bir::Bir::new(db, origin, BirData::new(tables, start_basic_block), origins)
}

impl Cursor {
    pub(crate) fn brew_expr_for_side_effects(
        &mut self,
        brewery: &mut Brewery<'_>,
        expr: validated::Expr,
    ) {
        let origin = brewery.origin(expr);
        match expr.data(brewery.validated_tables()) {
            validated::ExprData::Break {
                from_expr,
                with_value,
            } => {
                let loop_context = brewery.loop_context(*from_expr);
                self.brew_expr_and_assign_to(brewery, loop_context.loop_value, *with_value);
                self.terminate_and_goto(brewery, loop_context.break_block, origin);
            }

            validated::ExprData::Continue(from_expr) => {
                let loop_context = brewery.loop_context(*from_expr);
                self.terminate_and_goto(brewery, loop_context.continue_block, origin);
            }

            validated::ExprData::Return(value_expr) => {
                if let Some(value_place) = self.brew_expr_to_place(brewery, *value_expr) {
                    self.terminate_and_diverge(
                        brewery,
                        bir::TerminatorData::Return(value_place),
                        origin,
                    );
                }
            }

            validated::ExprData::Error => {
                self.terminate_and_diverge(brewery, bir::TerminatorData::Error, origin)
            }

            validated::ExprData::Place(_)
            | validated::ExprData::Await(_)
            | validated::ExprData::If(_, _, _)
            | validated::ExprData::Loop(_)
            | validated::ExprData::Seq(_)
            | validated::ExprData::Op(_, _, _)
            | validated::ExprData::Assign(_, _)
            | validated::ExprData::BooleanLiteral(_)
            | validated::ExprData::IntegerLiteral(_)
            | validated::ExprData::StringLiteral(_)
            | validated::ExprData::Call(_, _)
            | validated::ExprData::Share(_)
            | validated::ExprData::Lease(_)
            | validated::ExprData::Give(_)
            | validated::ExprData::Tuple(_)
            | validated::ExprData::Atomic(_) => {
                let _ = self.brew_expr_to_place(brewery, expr);
            }
        }
    }

    pub(crate) fn brew_expr_to_place(
        &mut self,
        brewery: &mut Brewery<'_>,
        expr: validated::Expr,
    ) -> Option<bir::Place> {
        let origin = brewery.origin(expr);
        match expr.data(brewery.validated_tables()) {
            // Place expressions compile to a potentially complex place
            validated::ExprData::Place(place) => Some(self.brew_place(brewery, *place)),

            // Other expressions spill into a temporary
            validated::ExprData::BooleanLiteral(_)
            | validated::ExprData::Loop(_)
            | validated::ExprData::IntegerLiteral(_)
            | validated::ExprData::StringLiteral(_)
            | validated::ExprData::Await(_)
            | validated::ExprData::Call(_, _)
            | validated::ExprData::Share(_)
            | validated::ExprData::Lease(_)
            | validated::ExprData::Give(_)
            | validated::ExprData::Tuple(_)
            | validated::ExprData::If(_, _, _)
            | validated::ExprData::Atomic(_)
            | validated::ExprData::Break { .. }
            | validated::ExprData::Continue(_)
            | validated::ExprData::Return(_)
            | validated::ExprData::Seq(_)
            | validated::ExprData::Op(_, _, _)
            | validated::ExprData::Assign(_, _)
            | validated::ExprData::Error => {
                let temp_place = add_temporary_place(brewery, origin);
                self.brew_expr_and_assign_to(brewery, temp_place, expr);
                Some(temp_place)
            }
        }
    }

    /// Compiles an expression down to the value it produces.
    ///
    /// Returns `None` if this is an expression (like `break`) that
    /// produces diverging control flow (and hence no value).
    pub(crate) fn brew_expr_and_assign_to(
        &mut self,
        brewery: &mut Brewery<'_>,
        target: bir::Place,
        expr: validated::Expr,
    ) {
        let origin = brewery.origin(expr);
        match expr.data(brewery.validated_tables()) {
            validated::ExprData::Place(_) => {
                if let Some(place) = self.brew_expr_to_place(brewery, expr) {
                    self.push_assignment(brewery, target, bir::ExprData::Place(place), origin);
                }
            }

            validated::ExprData::Await(future) => {
                if let Some(place) = self.brew_expr_to_place(brewery, *future) {
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
                }
            }

            validated::ExprData::If(condition, if_true, if_false) => {
                if let Some(condition_place) = self.brew_expr_to_place(brewery, *condition) {
                    let if_true_block = brewery.dummy_block(origin);
                    let if_false_block = brewery.dummy_block(origin);
                    let join_block = self.terminate_and_continue(
                        brewery,
                        |_| bir::TerminatorData::If(condition_place, if_true_block, if_false_block),
                        origin,
                    );

                    let mut if_true_cursor = self.with_end_block(if_false_block);
                    if_true_cursor.brew_expr_and_assign_to(brewery, target, *if_true);
                    if_true_cursor.terminate_and_goto(brewery, join_block, origin);

                    let mut if_false_cursor = self.with_end_block(if_false_block);
                    if_false_cursor.brew_expr_and_assign_to(brewery, target, *if_false);
                    if_false_cursor.terminate_and_goto(brewery, join_block, origin);
                }
            }

            validated::ExprData::Loop(body) => {
                let body_block = brewery.dummy_block(origin);
                let break_block = self.terminate_and_continue(
                    brewery,
                    |_| bir::TerminatorData::Goto(body_block),
                    origin,
                );

                let mut body_brewery = brewery.subbrewery();
                body_brewery.push_loop_context(
                    expr,
                    LoopContext {
                        continue_block: body_block,
                        break_block,
                        loop_value: target,
                    },
                );
                let mut body_cursor = self.with_end_block(body_block);
                body_cursor.brew_expr_for_side_effects(brewery, *body);
                body_cursor.terminate_and_diverge(
                    brewery,
                    bir::TerminatorData::Goto(body_block),
                    origin,
                );
            }

            validated::ExprData::Share(place) => {
                let place = self.brew_place(brewery, *place);
                self.push_assignment(brewery, target, bir::ExprData::Share(place), origin);
            }

            validated::ExprData::Lease(place) => {
                let place = self.brew_place(brewery, *place);
                self.push_assignment(brewery, target, bir::ExprData::Lease(place), origin);
            }

            validated::ExprData::Give(place) => {
                let place = self.brew_place(brewery, *place);
                self.push_assignment(brewery, target, bir::ExprData::Give(place), origin)
            }

            validated::ExprData::BooleanLiteral(value) => self.push_assignment(
                brewery,
                target,
                bir::ExprData::BooleanLiteral(*value),
                origin,
            ),

            validated::ExprData::IntegerLiteral(value) => self.push_assignment(
                brewery,
                target,
                bir::ExprData::IntegerLiteral(*value),
                origin,
            ),

            validated::ExprData::StringLiteral(value) => self.push_assignment(
                brewery,
                target,
                bir::ExprData::StringLiteral(*value),
                origin,
            ),

            validated::ExprData::Tuple(exprs) => {
                if let Some(values) = exprs
                    .iter()
                    .map(|expr| self.brew_expr_to_place(brewery, *expr))
                    .collect::<Option<Vec<_>>>()
                {
                    assert_eq!(values.len(), exprs.len());
                    self.push_assignment(brewery, target, bir::ExprData::Tuple(values), origin);
                }
            }

            validated::ExprData::Op(lhs, op, rhs) => {
                if let Some(lhs) = self.brew_expr_to_place(brewery, *lhs) {
                    if let Some(rhs) = self.brew_expr_to_place(brewery, *rhs) {
                        self.push_assignment(
                            brewery,
                            target,
                            bir::ExprData::Op(lhs, *op, rhs),
                            origin,
                        );
                    }
                }
            }

            validated::ExprData::Seq(exprs) => {
                if let Some((last_expr, prefix)) = exprs.split_last() {
                    for e in prefix {
                        self.brew_expr_for_side_effects(brewery, *e);
                    }
                    self.brew_expr_and_assign_to(brewery, target, *last_expr);
                } else {
                    self.push_assignment(brewery, target, bir::ExprData::Tuple(vec![]), origin);
                }
            }

            validated::ExprData::Assign(_, _) => {
                self.brew_expr_for_side_effects(brewery, expr);
                self.push_assignment(brewery, target, bir::ExprData::Tuple(vec![]), origin);
            }

            validated::ExprData::Call(func, args) => {
                if let Some(func_place) = self.brew_expr_to_place(brewery, *func) {
                    if let Some(bir_args) = args
                        .iter()
                        .map(|arg| self.brew_named_expr(brewery, *arg))
                        .collect::<Option<Vec<_>>>()
                    {
                        assert_eq!(bir_args.len(), args.len());
                        self.terminate_and_continue(
                            brewery,
                            |next_block| {
                                bir::TerminatorData::Assign(
                                    target,
                                    bir::TerminatorExpr::Call(func_place, bir_args),
                                    next_block,
                                )
                            },
                            origin,
                        );
                    }
                }
            }
            validated::ExprData::Atomic(_) => todo!(),

            validated::ExprData::Error
            | validated::ExprData::Return(_)
            | validated::ExprData::Continue(_)
            | validated::ExprData::Break { .. } => {
                self.brew_expr_for_side_effects(brewery, expr);
            }
        };
    }

    pub(crate) fn brew_place(
        &mut self,
        brewery: &mut Brewery<'_>,
        place: validated::Place,
    ) -> bir::Place {
        let origin = brewery.origin(place);
        match place.data(brewery.validated_tables()) {
            validated::PlaceData::LocalVariable(validated_var) => {
                let bir_var = brewery.variable(*validated_var);
                brewery.add(bir::PlaceData::LocalVariable(bir_var), origin)
            }
            validated::PlaceData::Function(function) => {
                brewery.add(bir::PlaceData::Function(*function), origin)
            }
            validated::PlaceData::Intrinsic(intrinsic) => {
                brewery.add(bir::PlaceData::Intrinsic(*intrinsic), origin)
            }
            validated::PlaceData::Class(class) => {
                brewery.add(bir::PlaceData::Class(*class), origin)
            }
            validated::PlaceData::Dot(base, field) => {
                let base = self.brew_place(brewery, *base);
                brewery.add(bir::PlaceData::Dot(base, *field), origin)
            }
        }
    }
}

fn add_temporary(brewery: &mut Brewery, origin: syntax::Expr) -> bir::LocalVariable {
    let temp = brewery.add(
        bir::LocalVariableData {
            name: None,
            storage_mode: StorageMode::Var,
        },
        origin,
    );
    temp
}

fn add_temporary_place(brewery: &mut Brewery, origin: syntax::Expr) -> bir::Place {
    let temporary_var = add_temporary(brewery, origin);
    brewery.add(bir::PlaceData::LocalVariable(temporary_var), origin)
}
