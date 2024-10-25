use std::future::Future;

use dada_ir_ast::{
    ast::{AstExpr, AstExprKind, BinaryOp, SpannedIdentifier},
    diagnostic::{Diagnostic, Err, Level, Reported},
    span::Span,
};
use dada_ir_sym::{
    function::SymFunction,
    prelude::IntoSymInScope,
    scope::{NameResolution, NameResolutionSym},
    symbol::{SymGenericKind, SymVariable},
    ty::{SymGenericTerm, SymTyName},
};
use dada_parser::prelude::*;
use dada_util::FromImpls;

use crate::{
    check::Check,
    env::Env,
    member::MemberLookup,
    object_ir::{
        IntoObjectIr, ObjectExpr, ObjectExprKind, ObjectPlaceExpr, ObjectPlaceExprKind, ObjectTy,
        ObjectTyKind,
    },
    Checking,
};

#[derive(Clone)]
pub(crate) struct ExprResult<'db> {
    /// List of [`Temporary`][] variables created by this expression.
    pub temporaries: Vec<Temporary<'db>>,

    /// Span of the expression
    pub span: Span<'db>,

    /// The primary result from translating an expression.
    /// Note that an ast-expr can result in many kinds of things.
    pub kind: ExprResultKind<'db>,
}

/// Translating an expression can result in the creation of
/// anonymous local temporaries that are injected into the
/// surrounding scope. These are returned alongside the result
/// and will eventually be translated into `let-in` expressions
/// when we reach the surrounding statement, block, or other
/// terminating context.
#[derive(Clone)]
pub(crate) struct Temporary<'db> {
    pub lv: SymVariable<'db>,
    pub ty: ObjectTy<'db>,
    pub initializer: Option<ObjectExpr<'db>>,
}

impl<'db> Temporary<'db> {
    pub fn new(
        db: &'db dyn crate::Db,
        span: Span<'db>,
        ty: ObjectTy<'db>,
        initializer: Option<ObjectExpr<'db>>,
    ) -> Self {
        let lv = SymVariable::new(db, SymGenericKind::Place, None, span);
        Self {
            lv,
            ty,
            initializer,
        }
    }
}

#[derive(Clone, Debug, FromImpls)]
pub(crate) enum ExprResultKind<'db> {
    /// An expression identifying a place in memory (e.g., a local variable).
    PlaceExpr(ObjectPlaceExpr<'db>),

    /// An expression that produces a value.
    Expr(ObjectExpr<'db>),

    /// A partially completed method call.
    #[no_from_impl]
    Method {
        self_expr: ObjectExpr<'db>,
        id_span: Span<'db>,
        function: SymFunction<'db>,
        generics: Option<Vec<SymGenericTerm<'db>>>,
    },

    /// Some kind of name resoluton that cannot be represented by as an expression.
    Other(NameResolution<'db>),
}

impl<'db> Checking<'db> for AstExpr<'db> {
    type Checking = ExprResult<'db>;

    fn check(&self, check: &Check<'db>, env: &Env<'db>) -> impl Future<Output = Self::Checking> {
        Box::pin(check_expr(self, check, env))
    }
}

async fn check_expr<'db>(
    expr: &AstExpr<'db>,
    check: &Check<'db>,
    env: &Env<'db>,
) -> ExprResult<'db> {
    let db = check.db;
    let scope = &env.scope;
    let span = expr.span;

    match &*expr.kind {
        AstExprKind::Literal(literal) => {
            let ty = env.fresh_object_ty_inference_var(check);
            check.defer(env, async move |check, env| todo!());
            ExprResult {
                temporaries: vec![],
                span,
                kind: ObjectExpr::new(db, span, ty, ObjectExprKind::Literal(literal.clone()))
                    .into(),
            }
        }

        AstExprKind::Tuple(span_vec) => {
            let mut temporaries = vec![];
            let mut exprs = vec![];
            for element in &span_vec.values {
                exprs.push(
                    element
                        .check(check, env)
                        .await
                        .into_expr(check, env, &mut temporaries),
                );
            }

            let ty = ObjectTy::new(
                db,
                ObjectTyKind::Named(
                    SymTyName::Tuple { arity: exprs.len() },
                    exprs.iter().map(|e| e.ty(db).into()).collect(),
                ),
            );

            ExprResult {
                temporaries,
                span,
                kind: ExprResultKind::Expr(ObjectExpr::new(
                    db,
                    span,
                    ty,
                    ObjectExprKind::Tuple(exprs),
                )),
            }
        }

        AstExprKind::BinaryOp(span_op, lhs, rhs) => {
            let lhs = lhs.check(check, env).await;
            let rhs = rhs.check(check, env).await;
            match span_op.op {
                BinaryOp::Add => todo!(),
                BinaryOp::Sub => todo!(),
                BinaryOp::Mul => todo!(),
                BinaryOp::Div => todo!(),
            }
        }

        AstExprKind::Id(SpannedIdentifier { span: id_span, id }) => {
            match env.scope.resolve_name(db, *id, *id_span) {
                Err(reported) => ExprResult::err(db, reported),
                Ok(res) => ExprResult::from_name_resolution(check, env, res, span),
            }
        }

        AstExprKind::DotId(owner, id) => {
            let mut owner_result = owner.check(check, env).await;
            match owner_result.kind {
                ExprResultKind::PlaceExpr(_) | ExprResultKind::Expr(_) => {
                    MemberLookup::new(check, &env)
                        .lookup_member(owner_result, *id)
                        .await
                }

                ExprResultKind::Other(name_resolution) => {
                    match name_resolution.resolve_relative_id(db, *id) {
                        Err(reported) => ExprResult::err(db, reported),
                        Ok(Ok(r)) => ExprResult::from_name_resolution(check, env, r, span),
                        Ok(Err(r)) => {
                            owner_result.kind = r.into();
                            MemberLookup::new(check, &env)
                                .lookup_member(owner_result, *id)
                                .await
                        }
                    }
                }

                ExprResultKind::Method {
                    self_expr: owner,
                    function: method,
                    ..
                } => ExprResult::err(
                    db,
                    report_missing_call_to_method(db, owner.span(db), method),
                ),
            }
        }

        AstExprKind::SquareBracketOp(owner, square_bracket_args) => {
            let owner_result = owner.check(check, env).await;
            match &owner_result.kind {
                &ExprResultKind::Method {
                    self_expr: owner,
                    function: method,
                    generics: None,
                    id_span,
                } => {
                    let ast_terms = square_bracket_args.parse_as_generics(db);

                    let sym_terms = ast_terms
                        .values
                        .iter()
                        .map(|ast_term| ast_term.into_sym_in_scope(db, scope))
                        .collect();

                    ExprResult {
                        kind: ExprResultKind::Method {
                            self_expr: owner,
                            function: method,
                            generics: Some(sym_terms),
                            id_span,
                        },
                        ..owner_result
                    }
                }

                ExprResultKind::PlaceExpr(_) | ExprResultKind::Expr(_) => {
                    ExprResult::err(db, report_not_implemented(db, span, "indexing expressions"))
                }

                // We see something like `foo.bar[][]` where `bar` is a method.
                // The only correct thing here would be `foo.bar[]()[]`, i.e., call the method and then index.
                // We give an error under that assumption.
                // It seems likely we can do a better job.
                &ExprResultKind::Method {
                    self_expr: owner,
                    function: method,
                    generics: Some(_),
                    ..
                } => ExprResult::err(db, report_missing_call_to_method(db, span, method)),

                ExprResultKind::Other(name_resolution) => {
                    ExprResult::err(db, report_non_expr(db, owner.span, name_resolution))
                }
            }
        }

        AstExprKind::ParenthesisOp(owner, ast_args) => {
            let owner_result = owner.check(check, env).await;
            match owner_result {
                ExprResult {
                    temporaries,
                    span: expr_span,
                    kind:
                        ExprResultKind::Method {
                            self_expr,
                            id_span,
                            function,
                            generics,
                        },
                } => {
                    check_call(
                        check,
                        env,
                        id_span,
                        expr_span,
                        function,
                        Some(self_expr),
                        ast_args,
                        generics,
                        temporaries,
                    )
                    .await
                }

                _ => {
                    // FIXME: we probably want to support functions and function typed values?
                    ExprResult::err(db, report_not_callable(db, span))
                }
            }
        }

        AstExprKind::Constructor(ast_path, span_vec) => todo!(),
        AstExprKind::Return(ast_expr) => {
            let mut temporaries = vec![];

            let return_expr = if let Some(ast_expr) = ast_expr {
                ast_expr
                    .check(check, env)
                    .await
                    .into_expr(check, env, &mut temporaries)
            } else {
                // the default is `return ()`
                ObjectExpr::new(db, span, ObjectTy::unit(db), ObjectExprKind::Tuple(vec![]))
            };

            let Some(expected_return_ty) = env.return_ty else {
                return ExprResult::err(
                    db,
                    Diagnostic::error(db, span, format!("unexpected `return` statement"))
                        .label(
                            db,
                            Level::Error,
                            span,
                            format!("I did not expect to see a `return` statement here"),
                        )
                        .report(db),
                );
            };

            env.require_assignable_object_type(
                check,
                return_expr.span(db),
                return_expr.ty(db),
                expected_return_ty,
            );

            ExprResult {
                temporaries,
                span,
                kind: ObjectExpr::new(
                    db,
                    span,
                    ObjectTy::never(db),
                    ObjectExprKind::Return(return_expr),
                )
                .into(),
            }
        }
    }
}

async fn check_call<'db>(
    check: &Check<'db>,
    env: &Env<'db>,
    id_span: Span<'db>,
    expr_span: Span<'db>,
    function: SymFunction<'db>,
    self_expr: Option<ObjectExpr<'db>>,
    ast_args: &[AstExpr<'db>],
    generics: Option<Vec<SymGenericTerm<'db>>>,
    mut temporaries: Vec<Temporary<'db>>,
) -> ExprResult<'db> {
    let db = check.db;

    // Get the signature.
    let signature = function.signature(db);
    let input_output = signature.input_output(db);

    // Instantiate the class generics with inference variables.
    // (FIXME: Is there a way for people to specify these explicitly?)
    let class_substitution = env.existential_substitution(check, input_output);
    let input_output = input_output.substitute(db, &class_substitution);

    // Instantiate the method generics with inference variables
    // or use the provided values (if any).
    let method_substitution = match generics {
        Some(generics) => {
            let expected_generics = input_output.len();
            let found_generics = generics.len();
            let function_name = function.name(db);
            if expected_generics != found_generics {
                return ExprResult::err(
                    db,
                    Diagnostic::error(
                        db,
                        id_span,
                        format!("expected {expected_generics} generic arguments, found {found_generics}"),
                    )
                    .label(
                        db,
                        Level::Error,
                        id_span,
                        format!("I expected `{function_name}` to take {expected_generics} arguments but I found {found_generics}",),
                    )
                    .label(
                        db,
                        Level::Info,
                        function.name_span(db),
                        format!("`{function_name}` defined here"),
                    )
                    .report(db),
                );
            }

            generics
        }
        None => env.existential_substitution(check, &input_output),
    };
    let input_ouput = input_output.substitute(db, &method_substitution);

    // Instantiate the first two levels of generics with inference variables.
    let input_output = env.open_existentially(check, signature.input_output(db));
    let input_output = env.open_existentially(check, &input_output);

    // Check the arity of the actual arguments.
    let expected_inputs = input_output.bound_value.input_tys.len();
    let found_inputs = ast_args.len();
    if ast_args.len() != expected_inputs {
        let function_name = function.name(db);
        return ExprResult::err(
            db,
            Diagnostic::error(
                db,
                id_span,
                format!("expected {expected_inputs} arguments, found {found_inputs}"),
            )
            .label(
                db,
                Level::Error,
                id_span,
                format!("I expected `{function_name}` to take {expected_inputs} arguments but I found {found_inputs}",),
            )
            .label(
                db,
                Level::Info,
                function.name_span(db),
                format!("`{function_name}` defined here"),
            )
            .report(db),
        );
    }

    // Create the temporaries that will hold the values for each argument.
    let arg_temp_span = |i: usize| ast_args.get(i).map(|a| a.span).unwrap_or(id_span);
    let arg_temp_symbols = (0..)
        .map(|i| SymVariable::new(db, SymGenericKind::Place, None, arg_temp_span(i)))
        .collect::<Vec<_>>();
    let arg_temp_terms = arg_temp_symbols
        .iter()
        .map(|&sym| SymGenericTerm::var(db, sym.kind(db), Var::Universal(sym)))
        .collect::<Vec<_>>();

    // Instantiate the final level of binding with those temporaries
    let input_output = input_output.substitute(db, &arg_temp_terms);

    // Function to type check a single argument and check it has the correct type.
    let check_arg = async |i: usize| -> ExprResult<'db> {
        let mut arg_temporaries = vec![];
        let ast_arg = &ast_args[i];
        let expr = ast_arg
            .check(check, env)
            .await
            .into_expr(check, env, &mut arg_temporaries);
        env.require_assignable_object_type(
            check,
            expr.span(db),
            expr.ty(db),
            input_output.input_tys[i],
        );
        ExprResult::from_expr(check, env, expr, arg_temporaries)
    };

    // Type check the arguments; these can proceed concurrently.
    let mut arg_exprs = vec![];
    arg_exprs.extend(self_expr);
    for arg_result in futures::future::join_all((0..ast_args.len()).map(check_arg)).await {
        arg_exprs.push(arg_result.into_expr(check, env, &mut temporaries));
    }

    // Create the resulting call, which always looks like
    //
    //     let tmp1 = arg1 in
    //     let tmp2 = arg2 in
    //     ...
    //     call(tmp1, tmp2, ...)
    let mut call_expr = ObjectExpr::new(
        db,
        expr_span,
        input_output.output_ty.into_object_ir(db),
        ObjectExprKind::Call {
            function,
            class_substitution,
            method_substitution,
            arg_temps: arg_temp_symbols.clone(),
        },
    );
    for (arg_temp_symbol, arg_expr) in arg_temp_symbols
        .into_iter()
        .rev()
        .zip(arg_exprs.into_iter().rev())
    {
        call_expr = ObjectExpr::new(
            db,
            call_expr.span(db),
            arg_expr.ty(db),
            ObjectExprKind::LetIn {
                lv: arg_temp_symbol,
                sym_ty: None,
                ty: arg_expr.ty(db),
                initializer: Some(arg_expr),
                body: call_expr,
            },
        );
    }

    // Create the final result.
    ExprResult::from_expr(check, env, call_expr, temporaries)
}

impl<'db> Err<'db> for ExprResult<'db> {
    fn err(db: &'db dyn salsa::Database, r: Reported) -> Self {
        Self {
            temporaries: vec![],
            span: r.span(db),
            kind: ExprResultKind::Expr(ObjectExpr::err(db, r)),
        }
    }
}

impl<'db> ExprResult<'db> {
    /// Create a result based on lexical name resolution.
    pub fn from_name_resolution(
        check: &Check<'db>,
        env: &Env<'db>,
        res: NameResolution<'db>,
        span: Span<'db>,
    ) -> Self {
        let db = check.db;
        match res.sym {
            NameResolutionSym::SymVariable(var) if var.kind(db) == SymGenericKind::Place => {
                let ty = env.variable_ty(var).into_object_ir(db);
                let place_expr = ObjectPlaceExpr::new(db, span, ty, ObjectPlaceExprKind::Var(var));
                Self {
                    temporaries: vec![],
                    span,
                    kind: ExprResultKind::PlaceExpr(place_expr),
                }
            }

            // FIXME: Should functions be expressions?
            NameResolutionSym::SymFunction(_)
            | NameResolutionSym::SymModule(_)
            | NameResolutionSym::SymClass(_)
            | NameResolutionSym::SymPrimitive(_)
            | NameResolutionSym::SymVariable(..) => Self {
                temporaries: vec![],
                span,
                kind: ExprResultKind::Other(res),
            },
        }
    }

    pub fn from_place_expr(
        check: &Check<'db>,
        env: &Env<'db>,
        expr: ObjectPlaceExpr<'db>,
        temporaries: Vec<Temporary<'db>>,
    ) -> Self {
        let db = check.db;
        Self {
            temporaries,
            span: expr.span(db),
            kind: ExprResultKind::PlaceExpr(expr),
        }
    }

    pub fn from_expr(
        check: &Check<'db>,
        env: &Env<'db>,
        expr: ObjectExpr<'db>,
        temporaries: Vec<Temporary<'db>>,
    ) -> Self {
        let db = check.db;
        Self {
            temporaries,
            span: expr.span(db),
            kind: ExprResultKind::Expr(expr),
        }
    }

    /// Convert this result into an expression, with `let ... in` statements inserted for temporaries.
    pub fn into_expr_with_enclosed_temporaries(
        self,
        check: &Check<'db>,
        env: &Env<'db>,
    ) -> ObjectExpr<'db> {
        let db = check.db;
        let mut temporaries = vec![];
        let mut expr = self.into_expr(check, env, &mut temporaries);
        for temporary in temporaries.into_iter().rev() {
            expr = ObjectExpr::new(
                db,
                expr.span(db),
                expr.ty(db),
                ObjectExprKind::LetIn {
                    lv: temporary.lv,
                    sym_ty: None,
                    ty: temporary.ty.into_object_ir(db),
                    initializer: temporary.initializer,
                    body: expr,
                },
            );
        }

        expr
    }

    /// Computes the type of this, treating it as an expression.
    /// Reports an error if this names something that cannot be made into an expression.
    pub fn ty(&self, check: &Check<'db>, env: &Env<'db>) -> ObjectTy<'db> {
        let db = check.db;
        match &self.kind {
            &ExprResultKind::PlaceExpr(place_expr) => place_expr.ty(db),
            &ExprResultKind::Expr(expr) => expr.ty(db),
            ExprResultKind::Other(name_resolution) => {
                ObjectTy::err(db, report_non_expr(db, self.span, name_resolution))
            }
            &ExprResultKind::Method {
                self_expr: owner,
                function: method,
                ..
            } => ObjectTy::err(
                db,
                report_missing_call_to_method(db, owner.span(db), method),
            ),
        }
    }

    pub fn into_place_expr(
        self,
        check: &Check<'db>,
        env: &Env<'db>,
        temporaries: &mut Vec<Temporary<'db>>,
    ) -> ObjectPlaceExpr<'db> {
        let db = check.db;
        temporaries.extend(self.temporaries);
        match self.kind {
            ExprResultKind::PlaceExpr(place_expr) => place_expr,

            // This is a value that needs to be stored in a temporary.
            ExprResultKind::Expr(expr) => {
                let ty = expr.ty(db);

                // Create a temporary to store the result of this expression.
                let temporary = Temporary::new(db, expr.span(db), expr.ty(db), Some(expr));
                let lv = temporary.lv;
                temporaries.push(temporary);

                // The result will be a reference to that temporary.
                ObjectPlaceExpr::new(db, self.span, ty, ObjectPlaceExprKind::Var(lv))
            }

            ExprResultKind::Other(name_resolution) => {
                let reported = report_non_expr(db, self.span, &name_resolution);
                ObjectPlaceExpr::err(db, reported)
            }

            ExprResultKind::Method {
                self_expr: owner,
                function: method,
                ..
            } => ObjectPlaceExpr::err(
                db,
                report_missing_call_to_method(db, owner.span(db), method),
            ),
        }
    }

    pub fn into_expr(
        self,
        check: &Check<'db>,
        env: &Env<'db>,
        temporaries: &mut Vec<Temporary<'db>>,
    ) -> ObjectExpr<'db> {
        let db = check.db;
        temporaries.extend(self.temporaries);
        match self.kind {
            ExprResultKind::Expr(expr) => expr,
            ExprResultKind::PlaceExpr(place_expr) => ObjectExpr::new(
                db,
                place_expr.span(db),
                place_expr.ty(db).shared(db),
                ObjectExprKind::Share(place_expr),
            ),
            ExprResultKind::Other(name_resolution) => {
                ObjectExpr::err(db, report_non_expr(db, self.span, &name_resolution))
            }
            ExprResultKind::Method {
                self_expr: owner,
                function: method,
                ..
            } => ObjectExpr::err(db, report_missing_call_to_method(db, self.span, method)),
        }
    }
}

fn report_not_implemented<'db>(db: &'db dyn crate::Db, span: Span<'db>, what: &str) -> Reported {
    Diagnostic::error(db, span, format!("not implemented yet :("))
        .label(
            db,
            Level::Error,
            span,
            format!("sorry, but {what} have not been implemented yet :(",),
        )
        .report(db)
}

fn report_non_expr<'db>(
    db: &'db dyn crate::Db,
    owner_span: Span<'db>,
    name_resolution: &NameResolution<'db>,
) -> Reported {
    Diagnostic::error(db, owner_span, format!("expected an expression"))
        .label(
            db,
            Level::Error,
            owner_span,
            format!(
                "I expected to find an expresison but I found {}",
                name_resolution.categorize(db),
            ),
        )
        .report(db)
}

fn report_missing_call_to_method<'db>(
    db: &'db dyn crate::Db,
    owner_span: Span<'db>,
    method: SymFunction<'db>,
) -> Reported {
    Diagnostic::error(db, owner_span, format!("missing call to method"))
        .label(
            db,
            Level::Error,
            owner_span,
            format!(
                "`{}` is a method but you don't appear to be calling it",
                method.name(db),
            ),
        )
        .label(db, Level::Help, owner_span.at_end(), "maybe add `()` here?")
        .report(db)
}

fn report_not_callable<'db>(db: &'db dyn crate::Db, owner_span: Span<'db>) -> Reported {
    Diagnostic::error(db, owner_span, format!("not callable"))
        .label(
            db,
            Level::Error,
            owner_span,
            format!("this is not something you can call"),
        )
        .report(db)
}
