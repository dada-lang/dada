use dada_id::prelude::*;
use dada_ir::code::syntax;
use dada_ir::code::syntax::LocalVariableDecl;
use dada_ir::code::validated;
use dada_ir::code::Code;
use dada_ir::diagnostic::ErrorReported;
use dada_ir::effect::Effect;
use dada_ir::kw::Keyword;
use dada_ir::origin_table::HasOriginIn;
use dada_ir::origin_table::PushOriginIn;
use dada_ir::span::FileSpan;
use dada_ir::span::Span;
use dada_ir::storage_mode::StorageMode;
use dada_ir::word::Word;
use dada_lex::prelude::*;
use dada_parse::prelude::*;
use std::rc::Rc;
use std::str::FromStr;

use super::name_lookup::Definition;
use super::name_lookup::Scope;

pub(crate) struct Validator<'me> {
    db: &'me dyn crate::Db,
    code: Code,
    syntax_tree: &'me syntax::TreeData,
    tables: &'me mut validated::Tables,
    origins: &'me mut validated::Origins,
    loop_stack: Vec<validated::Expr>,
    scope: Scope<'me>,
    effect: Effect,
    effect_span: Rc<dyn Fn(&Validator<'_>) -> FileSpan + 'me>,
}

impl<'me> Validator<'me> {
    pub(crate) fn new(
        db: &'me dyn crate::Db,
        code: Code,
        syntax_tree: syntax::Tree,
        tables: &'me mut validated::Tables,
        origins: &'me mut validated::Origins,
        scope: Scope<'me>,
        effect_span: impl Fn(&Validator<'_>) -> FileSpan + 'me,
    ) -> Self {
        let syntax_tree = syntax_tree.data(db);
        Self {
            db,
            code,
            syntax_tree,
            tables,
            origins,
            loop_stack: vec![],
            scope,
            effect: code.effect,
            effect_span: Rc::new(effect_span),
        }
    }

    fn subscope(&mut self) -> Validator<'_> {
        Validator {
            db: self.db,
            code: self.code,
            syntax_tree: self.syntax_tree,
            tables: self.tables,
            origins: self.origins,
            loop_stack: self.loop_stack.clone(),
            scope: self.scope.subscope(),
            effect: self.effect,
            effect_span: self.effect_span.clone(),
        }
    }

    fn effect_span(&self) -> FileSpan {
        (self.effect_span)(self)
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

    pub(crate) fn syntax_tables(&self) -> &'me syntax::Tables {
        &self.syntax_tree.tables
    }

    pub(crate) fn num_local_variables(&self) -> usize {
        usize::from(validated::LocalVariable::max_key(self.tables))
    }

    fn add<V, O>(&mut self, data: V, origin: O) -> V::Key
    where
        V: dada_id::InternValue<Table = validated::Tables>,
        V::Key: PushOriginIn<validated::Origins, Origin = O>,
    {
        let key = self.tables.add(data);
        self.origins.push(key, origin);
        key
    }

    fn or_error(
        &mut self,
        data: Result<validated::Expr, ErrorReported>,
        origin: syntax::Expr,
    ) -> validated::Expr {
        data.unwrap_or_else(|ErrorReported| self.add(validated::ExprData::Error, origin))
    }

    fn span(&self, e: impl HasOriginIn<syntax::Spans, Origin = Span>) -> FileSpan {
        self.code.syntax_tree(self.db).spans(self.db)[e].in_file(self.code.filename(self.db))
    }

    fn empty_tuple(&mut self, origin: syntax::Expr) -> validated::Expr {
        self.add(validated::ExprData::Tuple(vec![]), origin)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn validate_parameter(&mut self, decl: LocalVariableDecl) {
        let decl_data = decl.data(self.syntax_tables());
        let local_variable = self.add(
            validated::LocalVariableData {
                name: Some(decl_data.name),
                storage_mode: decl_data.mode.unwrap_or(StorageMode::Shared),
            },
            validated::LocalVariableOrigin::Parameter(decl),
        );
        self.scope.insert(decl_data.name, local_variable);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn validate_expr(&mut self, expr: syntax::Expr) -> validated::Expr {
        tracing::trace!("expr.data = {:?}", expr.data(self.syntax_tables()));
        match expr.data(self.syntax_tables()) {
            syntax::ExprData::Dot(..) | syntax::ExprData::Id(_) => {
                let place = self.validate_expr_as_place(expr);
                self.place_to_expr(place, expr)
            }

            syntax::ExprData::BooleanLiteral(b) => {
                self.add(validated::ExprData::BooleanLiteral(*b), expr)
            }

            syntax::ExprData::IntegerLiteral(w) => {
                let raw_str = w.as_str(self.db);
                let without_underscore: String = raw_str.chars().filter(|&c| c != '_').collect();
                match u64::from_str(&without_underscore) {
                    Ok(v) => self.add(validated::ExprData::IntegerLiteral(v), expr),
                    Err(e) => {
                        dada_ir::error!(
                            self.span(expr),
                            "`{}` is not a valid integer: {}",
                            w.as_str(self.db),
                            e,
                        )
                        .emit(self.db);
                        self.add(validated::ExprData::Error, expr)
                    }
                }
            }

            syntax::ExprData::StringLiteral(w) => {
                let word_str = w.as_str(self.db);
                let dada_string = convert_to_dada_string(word_str);
                let word = Word::from(self.db, dada_string);
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

                let validated_future_expr = self.validate_expr(*future_expr);
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
                self.validate_permission_expr(expr, *target_expr, validated::ExprData::Share)
            }

            syntax::ExprData::Lease(target_expr) => {
                self.validate_permission_expr(expr, *target_expr, validated::ExprData::Lease)
            }

            syntax::ExprData::Give(target_expr) => {
                self.validate_permission_expr(expr, *target_expr, validated::ExprData::Give)
            }

            syntax::ExprData::Var(decl, initializer_expr) => {
                let decl_data = decl.data(self.syntax_tables());
                let local_variable = self.add(
                    validated::LocalVariableData {
                        name: Some(decl_data.name),
                        storage_mode: decl_data.mode.unwrap_or(StorageMode::Shared),
                    },
                    validated::LocalVariableOrigin::LocalVariable(*decl),
                );
                let place = self.add(validated::PlaceData::LocalVariable(local_variable), expr);
                let validated_initializer_expr = self.validate_expr(*initializer_expr);
                self.scope.insert(decl_data.name, local_variable);
                self.add(
                    validated::ExprData::Assign(place, validated_initializer_expr),
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

            syntax::ExprData::If(condition_expr, then_expr, else_expr) => {
                let validated_condition_expr = self.validate_expr(*condition_expr);
                let validated_then_expr = self.validate_expr(*then_expr);
                let validated_else_expr = match else_expr {
                    None => self.empty_tuple(expr),
                    Some(else_expr) => self.validate_expr(*else_expr),
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

            syntax::ExprData::Atomic(atomic_expr) => {
                let mut subscope = self.subscope().with_effect(Effect::Atomic, |this| {
                    this.span(expr).leading_keyword(this.db, Keyword::Atomic)
                });
                let validated_atomic_expr = subscope.validate_expr(*atomic_expr);
                std::mem::drop(subscope);
                self.add(validated::ExprData::Atomic(validated_atomic_expr), expr)
            }

            syntax::ExprData::Loop(body_expr) => {
                // Create the `validated::Expr` up front with "Error" to start; we are going to replace this later
                // with the actual loop.
                let loop_expr = self.add(validated::ExprData::Error, expr);

                let mut subscope = self.subscope();
                subscope.loop_stack.push(loop_expr);
                let validated_body_expr = subscope.validate_expr(*body_expr);
                std::mem::drop(subscope);

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
                let mut subscope = self.subscope();
                subscope.loop_stack.push(loop_expr);
                let validated_body_expr = subscope.validate_expr(*body_expr);
                drop(subscope);

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
                self.add(
                    validated::ExprData::Op(validated_lhs_expr, *op, validated_rhs_expr),
                    expr,
                )
            }

            syntax::ExprData::OpEq(lhs_expr, op, rhs_expr) => {
                let result = try {
                    let (validated_opt_temp_expr, validated_lhs_place) =
                        self.validate_expr_as_place(*lhs_expr)?;
                    let validated_lhs_expr =
                        self.add(validated::ExprData::Place(validated_lhs_place), expr);
                    let validated_rhs_expr = self.validate_expr(*rhs_expr);
                    let validated_op_expr = self.add(
                        validated::ExprData::Op(validated_lhs_expr, *op, validated_rhs_expr),
                        expr,
                    );
                    let assign_expr = self.add(
                        validated::ExprData::Assign(validated_lhs_place, validated_op_expr),
                        expr,
                    );
                    self.maybe_seq(validated_opt_temp_expr, assign_expr, expr)
                };
                self.or_error(result, expr)
            }

            syntax::ExprData::Assign(lhs_expr, rhs_expr) => {
                let place = try {
                    let (validated_opt_temp_expr, validated_lhs_place) =
                        self.validate_expr_as_place(*lhs_expr)?;
                    let validated_rhs_expr = self.validate_expr(*rhs_expr);
                    let assign_expr = self.add(
                        validated::ExprData::Assign(validated_lhs_place, validated_rhs_expr),
                        expr,
                    );
                    self.maybe_seq(validated_opt_temp_expr, assign_expr, expr)
                };
                self.or_error(place, expr)
            }

            syntax::ExprData::Error => self.add(validated::ExprData::Error, expr),
            syntax::ExprData::Seq(exprs) => {
                let validated_exprs: Vec<_> =
                    exprs.iter().map(|expr| self.validate_expr(*expr)).collect();
                self.add(validated::ExprData::Seq(validated_exprs), expr)
            }
        }
    }

    fn maybe_seq(
        &mut self,
        expr1: Option<validated::Expr>,
        expr2: validated::Expr,
        origin: syntax::Expr,
    ) -> validated::Expr {
        if let Some(expr1) = expr1 {
            self.add(validated::ExprData::Seq(vec![expr1, expr2]), origin)
        } else {
            expr2
        }
    }

    fn place_to_expr(
        &mut self,
        data: Result<(Option<validated::Expr>, validated::Place), ErrorReported>,
        origin: syntax::Expr,
    ) -> validated::Expr {
        match data {
            Ok((opt_assign_expr, place)) => {
                let place_expr = self.add(validated::ExprData::Place(place), origin);
                self.maybe_seq(opt_assign_expr, place_expr, origin)
            }
            Err(ErrorReported) => self.add(validated::ExprData::Error, origin),
        }
    }

    fn validate_permission_expr(
        &mut self,
        perm_expr: syntax::Expr,
        target_expr: syntax::Expr,
        perm_variant: impl Fn(validated::Place) -> validated::ExprData,
    ) -> validated::Expr {
        let validated_data = try {
            let (opt_temporary_expr, place) = self.validate_expr_as_place(target_expr)?;
            let permission_expr = self.add(perm_variant(place), perm_expr);
            self.maybe_seq(opt_temporary_expr, permission_expr, perm_expr)
        };
        self.or_error(validated_data, perm_expr)
    }

    fn validate_expr_as_place(
        &mut self,
        expr: syntax::Expr,
    ) -> Result<(Option<validated::Expr>, validated::Place), ErrorReported> {
        match expr.data(self.syntax_tables()) {
            syntax::ExprData::Id(name) => Ok((
                None,
                match self.scope.lookup(*name) {
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
                        return Err(dada_ir::error!(
                            self.span(expr),
                            "can't find anything named `{}`",
                            name.as_str(self.db)
                        )
                        .emit(self.db))
                    }
                },
            )),
            syntax::ExprData::Dot(owner_expr, field) => {
                let (opt_temporary_expr, validated_owner_place) =
                    self.validate_expr_as_place(*owner_expr)?;
                Ok((
                    opt_temporary_expr,
                    self.add(
                        validated::PlaceData::Dot(validated_owner_place, *field),
                        expr,
                    ),
                ))
            }
            syntax::ExprData::Parenthesized(parenthesized_expr) => {
                self.validate_expr_as_place(*parenthesized_expr)
            }
            syntax::ExprData::Error => Err(ErrorReported),
            _ => {
                let (assign_expr, temporary_place) = self.validate_expr_in_temporary(expr);
                Ok((Some(assign_expr), temporary_place))
            }
        }
    }

    /// Given an expression E, create a new temporary variable V and return a `V = E` expression.
    fn validate_expr_in_temporary(
        &mut self,
        expr: syntax::Expr,
    ) -> (validated::Expr, validated::Place) {
        let local_variable = self.add(
            validated::LocalVariableData {
                name: None,
                storage_mode: StorageMode::Var,
            },
            validated::LocalVariableOrigin::Temporary(expr),
        );

        let validated_place = self.add(validated::PlaceData::LocalVariable(local_variable), expr);
        let validated_expr = self.validate_expr(expr);

        let assign_expr = self.add(
            validated::ExprData::Assign(validated_place, validated_expr),
            expr,
        );
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
        let syntax::NamedExprData { name, expr } = named_expr.data(self.syntax_tables());
        let validated_expr = self.validate_expr(*expr);
        self.add(
            validated::NamedExprData {
                name: *name,
                expr: validated_expr,
            },
            named_expr,
        )
    }
}

fn count_bytes_in_common(s1: &[u8], s2: &[u8]) -> usize {
    s1.iter().zip(s2).take_while(|(c1, c2)| c1 == c2).count()
}

// Remove leading, trailing whitespace and common indent from multiline strings.
fn convert_to_dada_string(s: &str) -> String {
    // If the string has only one line, leave it and return immediately.
    if s.lines().count() == 1 {
        return s.to_string();
    }

    // Split string into lines and filter out empty lines.
    let mut non_empty_line_iter = s.lines().filter(|&line| !line.trim().is_empty());

    if let Some(first_line) = non_empty_line_iter.next() {
        let prefix = first_line
            .chars()
            .into_iter()
            .take_while(|c| c.is_whitespace())
            .collect::<String>();
        let common_indent = non_empty_line_iter
            .map(|s| count_bytes_in_common(prefix.as_bytes(), s.as_bytes()))
            .min()
            .unwrap_or(0);

        // Remove the common indent from every line in the original string,
        // apart from empty lines, which remain as empty.
        let mut buf = vec![];
        for (i, line) in s.lines().enumerate() {
            if i > 0 {
                buf.push('\n');
            }
            if line.trim().is_empty() {
                buf.extend(line.chars());
            } else {
                buf.extend(line[common_indent..].chars());
            }
        }

        // Strip leading/trailing whitespace.
        return buf.into_iter().collect::<String>().trim().to_string();
    }
    String::new()
}
