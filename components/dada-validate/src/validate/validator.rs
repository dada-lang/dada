use dada_id::prelude::*;
use dada_ir::code::syntax;
use dada_ir::code::syntax::op::Op;
use dada_ir::code::validated;
use dada_ir::code::validated::ExprOrigin;
use dada_ir::code::validated::LocalVariableOrigin;
use dada_ir::diagnostic::ErrorReported;
use dada_ir::effect::Effect;
use dada_ir::function::Function;
use dada_ir::kw::Keyword;
use dada_ir::origin_table::HasOriginIn;
use dada_ir::origin_table::PushOriginIn;
use dada_ir::return_type::ReturnTypeKind;
use dada_ir::span::FileSpan;
use dada_ir::span::Span;
use dada_ir::storage::Atomic;
use dada_ir::word::Word;
use dada_lex::prelude::*;
use dada_parse::prelude::*;
use std::rc::Rc;
use std::str::FromStr;

use super::name_lookup::Definition;
use super::name_lookup::Scope;

mod string_literals;

pub(crate) struct Validator<'me> {
    db: &'me dyn crate::Db,
    function: Function,
    syntax_tree: syntax::Tree,
    pub(crate) syntax_tables: &'me syntax::Tables,
    tables: &'me mut validated::Tables,
    origins: &'me mut validated::Origins,
    loop_stack: Vec<validated::Expr>,
    scope: Scope<'me>,
    effect: Effect,
    effect_span: Rc<dyn Fn(&Validator<'_>) -> FileSpan + 'me>,
    synthesized: bool,
}

impl<'me> Validator<'me> {
    pub(crate) fn root(
        db: &'me dyn crate::Db,
        function: Function,
        syntax_tree: syntax::Tree,
        tables: &'me mut validated::Tables,
        origins: &'me mut validated::Origins,
        scope: Scope<'me>,
    ) -> Self {
        Self {
            db,
            function,
            syntax_tables: syntax_tree.tables(db),
            syntax_tree,
            tables,
            origins,
            loop_stack: vec![],
            scope,
            effect: function.effect(db),
            effect_span: Rc::new(move |_| function.effect_span(db)),
            synthesized: false,
        }
    }

    fn subscope(&mut self) -> Validator<'_> {
        Validator {
            db: self.db,
            function: self.function,
            syntax_tree: self.syntax_tree,
            syntax_tables: self.syntax_tables,
            tables: self.tables,
            origins: self.origins,
            loop_stack: self.loop_stack.clone(),
            scope: self.scope.subscope(),
            effect: self.effect,
            effect_span: self.effect_span.clone(),
            synthesized: self.synthesized,
        }
    }

    fn effect_span(&self) -> FileSpan {
        (self.effect_span)(self)
    }

    fn with_loop_expr(mut self, e: validated::Expr) -> Self {
        self.loop_stack.push(e);
        self
    }

    pub(crate) fn with_effect(
        mut self,
        effect: Effect,
        effect_span: impl Fn(&Validator<'_>) -> FileSpan + 'me,
    ) -> Self {
        self.effect = effect;
        self.effect_span = Rc::new(effect_span);
        self
    }

    pub(crate) fn num_local_variables(&self) -> usize {
        usize::from(validated::LocalVariable::max_key(self.tables))
    }

    fn add<V, O>(&mut self, data: V, origin: impl IntoOrigin<Origin = O>) -> V::Key
    where
        V: dada_id::InternValue<Table = validated::Tables>,
        V::Key: PushOriginIn<validated::Origins, Origin = O>,
    {
        let key = self.tables.add(data);
        self.origins
            .push(key, origin.maybe_synthesized(self.synthesized));
        key
    }

    fn span(&self, e: impl HasOriginIn<syntax::Spans, Origin = Span>) -> FileSpan {
        self.function.syntax_tree(self.db).spans(self.db)[e]
            .in_file(self.function.input_file(self.db))
    }

    fn empty_tuple(&mut self, origin: syntax::Expr) -> validated::Expr {
        self.add(validated::ExprData::Tuple(vec![]), origin)
    }

    pub(crate) fn validate_signature(&mut self, signature: &syntax::Signature) {
        // NB: The signature uses a distinct set of syntax tables.
        let syntax::Signature { tables, data, .. } = signature;
        for &lv in &data.parameters {
            let lv_data = &tables[lv];

            let local_variable = self.add(
                validated::LocalVariableData {
                    name: Some(lv_data.name),
                    atomic: lv_data.atomic,
                },
                validated::LocalVariableOrigin::Parameter(lv),
            );
            self.scope.insert(lv_data.name, local_variable);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn validate_root_expr(&mut self, expr: syntax::Expr) -> validated::Expr {
        let validated_expr = self.validate_expr(expr);
        if self.function.return_type(self.db).kind(self.db) == ReturnTypeKind::Value {
            if let validated::ExprData::Seq(exprs) = validated_expr.data(self.tables) {
                if exprs.is_empty() {
                    dada_ir::error!(
                        self.function.return_type(self.db).span(self.db),
                        "function body cannot be empty",
                    )
                    .primary_label("because function is supposed to return something")
                    .emit(self.db);
                }
            }
        } else {
            let origin = ExprOrigin::synthesized(expr);
            let unit = self.add(validated::ExprData::Tuple(vec![]), origin);
            if let validated::ExprData::Seq(exprs) = validated_expr.data_mut(self.tables) {
                exprs.push(unit);
            } else {
                return self.add(validated::ExprData::Seq(vec![validated_expr, unit]), origin);
            }
        }
        validated_expr
    }

    /// Given an expression `E`, takes ownership of its value:
    ///
    /// * If `E` is a place expression, like `a.b`, this is equivalent to `a.b.give`
    /// * If `E` is a value expression, like `foo()`, this just evaluates `E`
    ///
    /// The `move_expr` argument indicates the expression causing the move (e.g., the `give` expr
    /// or the `await` expr).
    #[tracing::instrument(level = "debug", skip(self, expr))]
    fn give_validated_expr(
        &mut self,
        moving_expr: syntax::Expr,
        expr: syntax::Expr,
    ) -> validated::Expr {
        if self.is_place_expression(expr) {
            self.validate_permission_expr(moving_expr, expr, validated::ExprData::Give)
        } else {
            self.validate_expr(expr)
        }
    }

    /// Validates an expression into a value:
    ///
    /// * If `E` is a place expression, like `a.b`, this is equivalent to `a.b.share`
    /// * If `E` is a value expression, like `foo()`, this just evaluates it
    #[tracing::instrument(level = "debug", skip(self, expr))]
    fn validate_expr(&mut self, expr: syntax::Expr) -> validated::Expr {
        tracing::trace!("expr.data = {:?}", expr.data(self.syntax_tables));
        match expr.data(self.syntax_tables) {
            syntax::ExprData::Dot(..) | syntax::ExprData::Id(_) => self
                .with_expr_validated_as_place(expr, &mut |this, place| {
                    this.add(validated::ExprData::Share(place), expr)
                }),

            syntax::ExprData::BooleanLiteral(b) => {
                self.add(validated::ExprData::BooleanLiteral(*b), expr)
            }

            syntax::ExprData::IntegerLiteral(w, suffix) => {
                let raw_str = w.as_str(self.db);
                let without_underscore: String = raw_str.chars().filter(|&c| c != '_').collect();
                let parse_error = |this: &mut Validator, e| {
                    dada_ir::error!(this.span(expr), "{}", e,).emit(this.db);
                    this.add(validated::ExprData::Error, expr)
                };
                match suffix {
                    Some(suffix) => {
                        let suffix_str = suffix.as_str(self.db);
                        match suffix_str {
                            "u" => match u64::from_str(&without_underscore) {
                                Ok(v) => {
                                    self.add(validated::ExprData::UnsignedIntegerLiteral(v), expr)
                                }
                                Err(e) => parse_error(
                                    self,
                                    format!(
                                        "`{}` is not a valid integer: {}",
                                        &without_underscore, e
                                    ),
                                ),
                            },
                            "i" => match i64::from_str(&without_underscore) {
                                Ok(v) => {
                                    self.add(validated::ExprData::SignedIntegerLiteral(v), expr)
                                }
                                Err(e) => parse_error(
                                    self,
                                    format!(
                                        "`{}` is not a valid integer: {}",
                                        &without_underscore, e
                                    ),
                                ),
                            },
                            _ => parse_error(
                                self,
                                format!("`{}` is not a valid integer suffxi", suffix_str),
                            ),
                        }
                    }
                    None => match u64::from_str(&without_underscore) {
                        Ok(v) => self.add(validated::ExprData::IntegerLiteral(v), expr),
                        Err(e) => parse_error(
                            self,
                            format!("`{}` is not a valid integer: {}", &without_underscore, e),
                        ),
                    },
                }
            }

            syntax::ExprData::FloatLiteral(w_int, w_frac) => {
                let raw_int_str = w_int.as_str(self.db);
                let raw_frac_str = w_frac.as_str(self.db);
                let int_chars = raw_int_str.chars();
                let frac_chars = raw_frac_str.chars();
                let all_chars = int_chars.chain(Some('.')).chain(frac_chars);
                let all_chars = all_chars.filter(|&c| c != '_');
                let full_str: String = all_chars.collect();
                match f64::from_str(&full_str) {
                    Ok(v) => self.add(validated::ExprData::FloatLiteral(eq_float::F64(v)), expr),
                    Err(e) => {
                        dada_ir::error!(
                            self.span(expr),
                            "`{}.{}` is not a valid float: {}",
                            w_int.as_str(self.db),
                            w_frac.as_str(self.db),
                            e,
                        )
                        .emit(self.db);
                        self.add(validated::ExprData::Error, expr)
                    }
                }
            }

            syntax::ExprData::StringLiteral(word) => {
                // Normally, StringLiterals are embedded within a `Concatenate` and they are
                // often converted directly by the code in `Self::concatenate`. But in the case
                // where no margin stripping is required, that code invokes this function,
                // and so our job is only to deal with escape sequences.
                let word_str = word.as_str(self.db);
                let escaped = self.support_escape(expr, word_str);
                let word = Word::intern(self.db, escaped);
                self.add(validated::ExprData::StringLiteral(word), expr)
            }

            syntax::ExprData::Await(future_expr) => {
                if !self.effect.permits_await() {
                    let await_span = self.span(expr).trailing_keyword(self.db, Keyword::Await);
                    match self.effect {
                        Effect::Atomic => {
                            dada_ir::error!(
                                await_span,
                                "await is not permitted inside atomic sections",
                            )
                            .primary_label("await is here")
                            .secondary_label(self.effect_span(), "atomic section entered here")
                            .emit(self.db);
                        }
                        Effect::Default => {
                            dada_ir::error!(
                                await_span,
                                "await is not permitted outside of async functions",
                            )
                            .primary_label("await is here")
                            .secondary_label(self.effect_span(), "fn not declared `async`")
                            .emit(self.db);
                        }
                        Effect::Async => {
                            unreachable!();
                        }
                    }
                }

                let validated_future_expr = self.give_validated_expr(expr, *future_expr);
                self.add(validated::ExprData::Await(validated_future_expr), expr)
            }

            syntax::ExprData::Call(func_expr, named_exprs) => {
                let validated_func_expr = self.validate_expr(*func_expr);
                let validated_named_exprs = self.validate_named_exprs(named_exprs);
                let mut name_required = false;
                for named_expr in &validated_named_exprs {
                    let name = named_expr.data(self.tables).name;
                    if name.word(self.db).is_some() {
                        name_required = true;
                    } else if name_required {
                        dada_ir::error!(name.span(self.db), "parameter name required",)
                            .primary_label("parameter name required here")
                            .emit(self.db);
                    }
                }

                self.add(
                    validated::ExprData::Call(validated_func_expr, validated_named_exprs),
                    expr,
                )
            }

            syntax::ExprData::Share(target_expr) => {
                if self.is_place_expression(*target_expr) {
                    self.validate_permission_expr(expr, *target_expr, validated::ExprData::Share)
                } else {
                    let validated_target_expr = self.validate_expr(*target_expr);
                    self.add(validated::ExprData::IntoShared(validated_target_expr), expr)
                }
            }

            syntax::ExprData::Lease(target_expr) => {
                self.validate_permission_expr(expr, *target_expr, validated::ExprData::Lease)
            }

            syntax::ExprData::Give(target_expr) => self.give_validated_expr(expr, *target_expr),

            syntax::ExprData::Var(decl, initializer_expr) => {
                let decl_data = decl.data(self.syntax_tables);
                let local_variable = self.add(
                    validated::LocalVariableData {
                        name: Some(decl_data.name),
                        atomic: decl_data.atomic,
                    },
                    validated::LocalVariableOrigin::LocalVariable(*decl),
                );
                self.scope.insert(decl_data.name, local_variable);

                let target_place = self.add(
                    validated::TargetPlaceData::LocalVariable(local_variable),
                    expr.synthesized(),
                );

                let validated_initializer_expr = self.validate_expr(*initializer_expr);

                self.add(
                    validated::ExprData::Assign(target_place, validated_initializer_expr),
                    expr,
                )
            }

            syntax::ExprData::Parenthesized(parenthesized_expr) => {
                self.validate_expr(*parenthesized_expr)
            }

            syntax::ExprData::Tuple(element_exprs) => {
                let validated_exprs = element_exprs
                    .iter()
                    .map(|expr| self.validate_expr(*expr))
                    .collect();
                self.add(validated::ExprData::Tuple(validated_exprs), expr)
            }

            syntax::ExprData::Concatenate(exprs) => self.concatenate(expr, exprs),

            syntax::ExprData::If(condition_expr, then_expr, else_expr) => {
                let validated_condition_expr = self.validate_expr(*condition_expr);
                let validated_then_expr = self.subscope().validate_expr_and_exit(*then_expr);
                let validated_else_expr = match else_expr {
                    None => self.empty_tuple(expr),
                    Some(else_expr) => self.subscope().validate_expr_and_exit(*else_expr),
                };
                self.add(
                    validated::ExprData::If(
                        validated_condition_expr,
                        validated_then_expr,
                        validated_else_expr,
                    ),
                    expr,
                )
            }

            syntax::ExprData::Atomic(atomic_keyword, atomic_expr) => {
                let validated_atomic_expr = self
                    .subscope()
                    .with_effect(Effect::Atomic, |this| this.span(*atomic_keyword))
                    .validate_expr_and_exit(*atomic_expr);
                self.add(validated::ExprData::Atomic(validated_atomic_expr), expr)
            }

            syntax::ExprData::Loop(body_expr) => {
                // Create the `validated::Expr` up front with "Error" to start; we are going to replace this later
                // with the actual loop.
                let loop_expr = self.add(validated::ExprData::Error, expr);

                let validated_body_expr = self
                    .subscope()
                    .with_loop_expr(loop_expr)
                    .validate_expr_and_exit(*body_expr);

                self.tables[loop_expr] = validated::ExprData::Loop(validated_body_expr);

                loop_expr
            }

            syntax::ExprData::While(condition_expr, body_expr) => {
                // while C { E }
                //
                // lowers to
                //
                // loop { E; if C {} else {break} }

                let loop_expr = self.add(validated::ExprData::Error, expr);

                // lower the condition C
                let validated_condition_expr = self.validate_expr(*condition_expr);

                // lower the body E, in a subscope so that `break` breaks out from `loop_expr`
                let validated_body_expr = self
                    .subscope()
                    .with_loop_expr(loop_expr)
                    .validate_expr_and_exit(*body_expr);

                let if_break_expr = {
                    // break
                    let empty_tuple = self.empty_tuple(expr);
                    let break_expr = self.add(
                        validated::ExprData::Break {
                            from_expr: loop_expr,
                            with_value: empty_tuple,
                        },
                        expr,
                    );

                    //
                    self.add(
                        validated::ExprData::If(validated_condition_expr, empty_tuple, break_expr),
                        expr,
                    )
                };

                // replace `loop_expr` contents with the loop body `{E; if C {} else break}`
                let loop_body = self.add(
                    validated::ExprData::Seq(vec![validated_body_expr, if_break_expr]),
                    expr,
                );
                self.tables[loop_expr] = validated::ExprData::Loop(loop_body);

                loop_expr
            }

            syntax::ExprData::Op(lhs_expr, op, rhs_expr) => {
                let validated_lhs_expr = self.validate_expr(*lhs_expr);
                let validated_rhs_expr = self.validate_expr(*rhs_expr);
                let validated_op = self.validated_op(*op);
                self.add(
                    validated::ExprData::Op(validated_lhs_expr, validated_op, validated_rhs_expr),
                    expr,
                )
            }

            syntax::ExprData::Unary(op, rhs_expr) => {
                let validated_rhs_expr = self.validate_expr(*rhs_expr);
                let validated_op = self.validated_op(*op);
                self.add(
                    validated::ExprData::Unary(validated_op, validated_rhs_expr),
                    expr,
                )
            }

            syntax::ExprData::OpEq(..) => self.validate_op_eq(expr),

            syntax::ExprData::Assign(lhs_expr, rhs_expr) => {
                self.validate_assign_expr(expr, *lhs_expr, *rhs_expr)
            }

            syntax::ExprData::Error => self.add(validated::ExprData::Error, expr),
            syntax::ExprData::Seq(exprs) => {
                let validated_exprs: Vec<_> =
                    exprs.iter().map(|expr| self.validate_expr(*expr)).collect();
                self.add(validated::ExprData::Seq(validated_exprs), expr)
            }
            syntax::ExprData::Continue => {
                let validated_data = match self.loop_stack.last() {
                    Some(loop_expr) => validated::ExprData::Continue(*loop_expr),
                    None => {
                        dada_ir::error!(self.span(expr), "cannot `continue` outside of a loop")
                            .primary_label("`continue` outside of a loop here")
                            .emit(self.db);
                        validated::ExprData::Error
                    }
                };
                self.add(validated_data, expr)
            }
            syntax::ExprData::Break(with_value) => {
                let validated_expr = if let Some(break_expr) = with_value {
                    self.validate_expr(*break_expr)
                } else {
                    self.empty_tuple(expr)
                };

                let validated_data = match self.loop_stack.last() {
                    Some(loop_expr) => validated::ExprData::Break {
                        from_expr: *loop_expr,
                        with_value: validated_expr,
                    },
                    None => {
                        dada_ir::error!(self.span(expr), "cannot `break` outside of a loop")
                            .primary_label("`break` outside of a loop here")
                            .emit(self.db);
                        validated::ExprData::Error
                    }
                };
                self.add(validated_data, expr)
            }
            syntax::ExprData::Return(with_value) => {
                match (self.function.return_type(self.db).kind(self.db), with_value) {
                    (ReturnTypeKind::Value, None) => {
                        dada_ir::error!(self.span(expr), "return requires an expression")
                            .primary_label(
                                "cannot just have `return` without an expression afterwards",
                            )
                            .secondary_label(
                                self.function.return_type(self.db).span(self.db),
                                "because the function returns a value",
                            )
                            .emit(self.db);
                    }
                    (ReturnTypeKind::Unit, Some(return_expr)) => {
                        dada_ir::error!(
                            self.span(*return_expr),
                            "cannot return a value in this function"
                        )
                        .primary_label("can only write `return` (without a value) in this function")
                        .secondary_label(
                            self.function.return_type(self.db).span(self.db),
                            "because function doesn't have `->` here",
                        )
                        .emit(self.db);
                    }
                    _ => {}
                }
                let validated_expr = if let Some(return_expr) = with_value {
                    self.validate_expr(*return_expr)
                } else {
                    self.empty_tuple(expr)
                };
                self.add(validated::ExprData::Return(validated_expr), expr)
            }
        }
    }

    fn validate_op_eq(&mut self, op_eq_expr: syntax::Expr) -> validated::Expr {
        // if user wrote `x += <rhs>`, we generate
        //
        // {
        //     temp_value = x + <rhs>
        //     x = temp2
        // }
        //
        // if user wrote `owner.field += <rhs>`, we generate
        //
        // {
        //     temp_leased_owner = owner.lease
        //     temp_value = <rhs>
        //     temp_value = temp_leased_owner.x + <rhs>
        //     temp_leased_owner.x = temp_value.give
        // }
        //
        // below, we will leave comments for the more complex version.

        let syntax::ExprData::OpEq(lhs_expr, op, rhs_expr) = self.syntax_tables[op_eq_expr] else {
            panic!("validated_op_eq invoked on something that was not an op-eq expr")
        };

        // `temp_leased_owner = owner.lease` (if this is a field)
        self.with_expr_validated_as_target_place(lhs_expr, &mut |this, validated_target_place| {
            this.validate_op_eq_with_target_place(
                op_eq_expr,
                lhs_expr,
                op,
                rhs_expr,
                validated_target_place,
            )
        })
    }

    fn validate_op_eq_with_target_place(
        &mut self,
        op_eq_expr: syntax::Expr,
        lhs_expr: syntax::Expr,
        op: Op,
        rhs_expr: syntax::Expr,
        validated_target_place: validated::TargetPlace,
    ) -> validated::Expr {
        // `temp_value = x + <rhs>` or `temp_value = temp_leased_owner.x + <rhs>`
        let (temporary_assign_expr, temporary_place) = {
            let validated_op = self.validated_op(op);

            // `x` or `temp_leased_owner.x`
            let validated_lhs_expr = {
                let lhs_place = match self.tables[validated_target_place] {
                    validated::TargetPlaceData::LocalVariable(lv) => self.add(
                        validated::PlaceData::LocalVariable(lv),
                        lhs_expr.synthesized(),
                    ),

                    validated::TargetPlaceData::Dot(owner, field) => self.add(
                        validated::PlaceData::Dot(owner, field),
                        lhs_expr.synthesized(),
                    ),
                };
                self.add(validated::ExprData::Give(lhs_place), lhs_expr.synthesized())
            };

            // <rhs>
            let validated_rhs_expr = self.validate_expr(rhs_expr);

            // `x + <rhs>` or `temp_leased_owner.x + <rhs>`
            let validated_op_expr = self.add(
                validated::ExprData::Op(validated_lhs_expr, validated_op, validated_rhs_expr),
                op_eq_expr.synthesized(),
            );

            self.store_validated_expr_in_temporary(validated_op_expr)
        };

        //
        let temporary_give = self.add(
            validated::ExprData::Give(temporary_place),
            op_eq_expr.synthesized(),
        );

        // `x = temp_value` or `temp_leased_owner.x = temp_value`
        let assign_field_expr = self.add(
            validated::ExprData::Assign(validated_target_place, temporary_give),
            op_eq_expr,
        );

        self.seq(Some(temporary_assign_expr), assign_field_expr)
    }

    fn with_expr_validated_as_target_place(
        &mut self,
        expr: syntax::Expr,
        op: &mut dyn FnMut(&mut Self, validated::TargetPlace) -> validated::Expr,
    ) -> validated::Expr {
        match expr.data(self.syntax_tables) {
            syntax::ExprData::Dot(owner, field_name) => {
                self.with_expr_validated_as_place(*owner, &mut |this, owner_place| {
                    let target_place = this.add(
                        validated::TargetPlaceData::Dot(owner_place, *field_name),
                        expr,
                    );
                    op(this, target_place)
                })
            }

            syntax::ExprData::Id(name) => match self.scope.lookup(*name) {
                Some(Definition::LocalVariable(lv)) => {
                    let place = self.add(validated::TargetPlaceData::LocalVariable(lv), expr);
                    op(self, place)
                }

                Some(definition @ Definition::Function(_))
                | Some(definition @ Definition::Class(_))
                | Some(definition @ Definition::Intrinsic(_)) => {
                    let ErrorReported = dada_ir::error!(
                        self.span(expr),
                        "you can only assign to local variables or fields, not {} like `{}`",
                        definition.plural_description(),
                        name.as_str(self.db),
                    )
                    .emit(self.db);
                    self.add(validated::ExprData::Error, expr)
                }

                None => {
                    let ErrorReported = dada_ir::error!(
                        self.span(expr),
                        "can't find anything named `{}`",
                        name.as_str(self.db)
                    )
                    .emit(self.db);
                    self.add(validated::ExprData::Error, expr)
                }
            },

            syntax::ExprData::Parenthesized(target_expr) => {
                self.with_expr_validated_as_target_place(*target_expr, op)
            }

            _ => {
                let _ = self.validate_expr(expr);
                let ErrorReported = dada_ir::error!(
                    self.span(expr),
                    "you can only assign to local variables and fields, not arbitrary expressions",
                )
                .emit(self.db);
                self.add(validated::ExprData::Error, expr)
            }
        }
    }

    /// Validate the expression and then exit the subscope (consumes self).
    /// See [`Self::exit`].
    fn validate_expr_and_exit(mut self, expr: syntax::Expr) -> validated::Expr {
        let validated_expr = self.validate_expr(expr);
        self.exit(validated_expr)
    }

    /// Exit the subscope (consumes self) and produce `validated_expr`
    /// (possibly wrapped in a `Declare` with any variables that were
    /// declared within this scope).
    ///
    /// Exiting the subscope will pop-off any variables that were declared
    /// within.
    ///
    /// Returns the validated result, wrapped in `Declare` if necessary.
    fn exit(mut self, validated_expr: validated::Expr) -> validated::Expr {
        let vars = self.scope.take_inserted();
        if vars.is_empty() {
            return validated_expr;
        }

        let origin = self.origins[validated_expr].synthesized();
        self.add(validated::ExprData::Declare(vars, validated_expr), origin)
    }

    /// Creates a sequence that first executes `exprs` (if any) and then `final_expr`,
    /// taking its final result from `final_expr`. Commonly used to combine
    /// an initializer for an (optional) temporary followed by code that uses the
    /// temporary (e.g., `t = 22; t + u`).
    fn seq(
        &mut self,
        exprs: impl IntoIterator<Item = validated::Expr>,
        final_expr: validated::Expr,
    ) -> validated::Expr {
        let mut exprs: Vec<validated::Expr> = exprs.into_iter().collect();
        if exprs.is_empty() {
            final_expr
        } else {
            let origin = self.origins[final_expr].synthesized();
            exprs.push(final_expr);
            self.add(validated::ExprData::Seq(exprs), origin)
        }
    }

    fn validate_permission_expr(
        &mut self,
        perm_expr: syntax::Expr,
        target_expr: syntax::Expr,
        perm_variant: impl Fn(validated::Place) -> validated::ExprData,
    ) -> validated::Expr {
        self.with_expr_validated_as_place(target_expr, &mut |this, place| {
            this.add(perm_variant(place), perm_expr)
        })
    }

    fn is_place_expression(&self, expr: syntax::Expr) -> bool {
        match expr.data(self.syntax_tables) {
            syntax::ExprData::Id(_) | syntax::ExprData::Dot(..) => true,
            syntax::ExprData::Parenthesized(parenthesized_expr) => {
                self.is_place_expression(*parenthesized_expr)
            }
            _ => false,
        }
    }

    /// Validates `expr` as a place. Invokes `op` with the place to
    /// produce the final expression. Doesn't directly return that
    /// expression because it may introduce temporaries
    /// that are referenced by the place given to op, and if `expr`
    /// is malformed may return an error expression.
    fn with_expr_validated_as_place(
        &mut self,
        expr: syntax::Expr,
        op: &mut dyn FnMut(&mut Self, validated::Place) -> validated::Expr,
    ) -> validated::Expr {
        match expr.data(self.syntax_tables) {
            syntax::ExprData::Id(name) => {
                let place = match self.scope.lookup(*name) {
                    Some(Definition::Class(c)) => self.add(validated::PlaceData::Class(c), expr),
                    Some(Definition::Function(f)) => {
                        self.add(validated::PlaceData::Function(f), expr)
                    }
                    Some(Definition::LocalVariable(lv)) => {
                        self.add(validated::PlaceData::LocalVariable(lv), expr)
                    }
                    Some(Definition::Intrinsic(i)) => {
                        self.add(validated::PlaceData::Intrinsic(i), expr)
                    }
                    None => {
                        let ErrorReported = dada_ir::error!(
                            self.span(expr),
                            "can't find anything named `{}`",
                            name.as_str(self.db)
                        )
                        .emit(self.db);
                        return self.add(validated::ExprData::Error, expr);
                    }
                };
                op(self, place)
            }
            syntax::ExprData::Dot(owner_expr, field) => {
                self.with_expr_validated_as_place(*owner_expr, &mut |this, owner_place| {
                    let dot_place = this.add(validated::PlaceData::Dot(owner_place, *field), expr);
                    op(this, dot_place)
                })
            }
            syntax::ExprData::Parenthesized(parenthesized_expr) => {
                self.with_expr_validated_as_place(*parenthesized_expr, op)
            }
            syntax::ExprData::Error => self.add(validated::ExprData::Error, expr),
            _ => {
                let (assign_expr, temporary_place) = self.validate_expr_in_temporary(expr);
                let expr = op(self, temporary_place);
                self.seq(Some(assign_expr), expr)
            }
        }
    }

    /// Given an expression E, create a new temporary variable V and return a `V = E` expression.
    fn validate_expr_in_temporary(
        &mut self,
        expr: syntax::Expr,
    ) -> (validated::Expr, validated::Place) {
        let validated_expr = self.validate_expr(expr);
        self.store_validated_expr_in_temporary(validated_expr)
    }

    /// Creates a temporary to store the result of validating some expression.
    fn store_validated_expr_in_temporary(
        &mut self,
        validated_expr: validated::Expr,
    ) -> (validated::Expr, validated::Place) {
        let origin = self.origins[validated_expr].synthesized();

        let local_variable = self.add(
            validated::LocalVariableData {
                name: None,
                atomic: Atomic::No,
            },
            validated::LocalVariableOrigin::Temporary(origin.syntax_expr),
        );
        self.scope.insert_temporary(local_variable);

        let target_place = self.add(
            validated::TargetPlaceData::LocalVariable(local_variable),
            origin,
        );
        let assign_expr = self.add(
            validated::ExprData::Assign(target_place, validated_expr),
            origin,
        );

        let validated_place = self.add(validated::PlaceData::LocalVariable(local_variable), origin);
        (assign_expr, validated_place)
    }

    fn validate_named_exprs(
        &mut self,
        named_exprs: &[syntax::NamedExpr],
    ) -> Vec<validated::NamedExpr> {
        named_exprs
            .iter()
            .map(|named_expr| self.validate_named_expr(*named_expr))
            .collect()
    }

    fn validate_named_expr(&mut self, named_expr: syntax::NamedExpr) -> validated::NamedExpr {
        let syntax::NamedExprData { name, expr } = named_expr.data(self.syntax_tables);
        let validated_expr = self.validate_expr(*expr);
        self.add(
            validated::NamedExprData {
                name: *name,
                expr: validated_expr,
            },
            named_expr,
        )
    }

    fn validated_op(&self, op: syntax::op::Op) -> validated::op::Op {
        match op {
            // Compound binops become a binop + assignment
            syntax::op::Op::PlusEqual => validated::op::Op::Plus,
            syntax::op::Op::MinusEqual => validated::op::Op::Minus,
            syntax::op::Op::TimesEqual => validated::op::Op::Times,
            syntax::op::Op::DividedByEqual => validated::op::Op::DividedBy,

            // Binops
            syntax::op::Op::EqualEqual => validated::op::Op::EqualEqual,
            syntax::op::Op::GreaterEqual => validated::op::Op::GreaterEqual,
            syntax::op::Op::LessEqual => validated::op::Op::LessEqual,
            syntax::op::Op::Plus => validated::op::Op::Plus,
            syntax::op::Op::Minus => validated::op::Op::Minus,
            syntax::op::Op::Times => validated::op::Op::Times,
            syntax::op::Op::DividedBy => validated::op::Op::DividedBy,
            syntax::op::Op::LessThan => validated::op::Op::LessThan,
            syntax::op::Op::GreaterThan => validated::op::Op::GreaterThan,

            // These are parsed into other syntax elements and should not appear
            // at this stage of compilation.
            syntax::op::Op::ColonEqual
            | syntax::op::Op::Colon
            | syntax::op::Op::SemiColon
            | syntax::op::Op::LeftAngle
            | syntax::op::Op::RightAngle
            | syntax::op::Op::Dot
            | syntax::op::Op::Equal
            | syntax::op::Op::RightArrow => {
                unreachable!("unexpected op")
            }
        }
    }

    pub(crate) fn validate_assign_expr(
        &mut self,
        assign_expr: syntax::Expr,
        lhs_expr: syntax::Expr,
        initializer_expr: syntax::Expr,
    ) -> validated::Expr {
        self.with_expr_validated_as_target_place(lhs_expr, &mut |this, target_place| {
            let validated_expr = this.validate_expr(initializer_expr);
            this.add(
                validated::ExprData::Assign(target_place, validated_expr),
                assign_expr,
            )
        })
    }
}

fn count_bytes_in_common(s1: &[u8], s2: &[u8]) -> usize {
    s1.iter().zip(s2).take_while(|(c1, c2)| c1 == c2).count()
}

trait IntoOrigin: Sized {
    type Origin;

    fn into_origin(self) -> Self::Origin;

    fn maybe_synthesized(self, s: bool) -> Self::Origin {
        if s {
            self.synthesized()
        } else {
            self.into_origin()
        }
    }

    fn synthesized(self) -> Self::Origin;
}

impl IntoOrigin for syntax::Expr {
    type Origin = ExprOrigin;

    fn into_origin(self) -> Self::Origin {
        ExprOrigin::real(self)
    }

    fn synthesized(self) -> Self::Origin {
        ExprOrigin::synthesized(self)
    }
}

impl IntoOrigin for syntax::NamedExpr {
    type Origin = syntax::NamedExpr;

    fn into_origin(self) -> Self::Origin {
        self
    }

    fn synthesized(self) -> Self::Origin {
        panic!("cannot force named expr origin to be synthesized")
    }
}

impl IntoOrigin for ExprOrigin {
    type Origin = ExprOrigin;

    fn into_origin(self) -> Self::Origin {
        self
    }

    fn synthesized(self) -> Self::Origin {
        ExprOrigin::synthesized(self.syntax_expr)
    }
}

impl IntoOrigin for LocalVariableOrigin {
    type Origin = LocalVariableOrigin;

    fn into_origin(self) -> Self::Origin {
        self
    }

    fn synthesized(self) -> Self::Origin {
        match self {
            // temporaries are synthesized local variables, so that's ok
            LocalVariableOrigin::Temporary(_) => self,

            // we can't make other variables be synthesized
            _ => panic!("cannot force local variable origin to be synthesized"),
        }
    }
}
