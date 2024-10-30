use std::future::Future;

use dada_ir_ast::{
    ast::{
        AstExpr, AstExprKind, AstGenericTerm, Identifier, LiteralKind, SpanVec, SpannedBinaryOp,
        SpannedIdentifier,
    },
    diagnostic::{Diagnostic, Err, Level, Reported},
    span::{Span, Spanned},
};
use dada_ir_sym::{
    binder::Binder,
    class::SymClass,
    function::{SymFunction, SymInputOutput},
    prelude::IntoSymInScope,
    primitive::{SymPrimitive, SymPrimitiveKind},
    scope::{NameResolution, NameResolutionSym},
    scope_tree::ScopeTreeNode,
    symbol::{FromVar, HasKind, SymGenericKind, SymVariable},
    ty::{SymGenericTerm, SymTyName},
};
use dada_parser::prelude::*;
use dada_util::FromImpls;
use futures::StreamExt;

use crate::{
    bound::Bound,
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
        generics: Option<SpanVec<'db, AstGenericTerm<'db>>>,
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
    let expr_span = expr.span;

    match &*expr.kind {
        AstExprKind::Literal(literal) => match literal.kind(db) {
            LiteralKind::Integer => {
                let ty = env.fresh_object_ty_inference_var(check);
                env.require_numeric_type(check, expr_span, ty);
                ExprResult {
                    temporaries: vec![],
                    span: expr_span,
                    kind: ObjectExpr::new(
                        db,
                        expr_span,
                        ty,
                        ObjectExprKind::Literal(literal.clone()),
                    )
                    .into(),
                }
            }
            LiteralKind::String => {
                // FIXME: strings should be in the stdlib I think
                let ty = SymPrimitive::new(db, SymPrimitiveKind::Str).into_object_ir(db);
                ExprResult {
                    temporaries: vec![],
                    span: expr_span,
                    kind: ObjectExpr::new(
                        db,
                        expr_span,
                        ty,
                        ObjectExprKind::Literal(literal.clone()),
                    )
                    .into(),
                }
            }
        },

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
                span: expr_span,
                kind: ExprResultKind::Expr(ObjectExpr::new(
                    db,
                    expr_span,
                    ty,
                    ObjectExprKind::Tuple(exprs),
                )),
            }
        }

        AstExprKind::BinaryOp(span_op, lhs, rhs) => {
            let span_op: SpannedBinaryOp<'db> = *span_op;

            let mut temporaries: Vec<Temporary<'db>> = vec![];
            let lhs: ObjectExpr<'db> =
                lhs.check(check, env)
                    .await
                    .into_expr(check, env, &mut temporaries);
            let rhs: ObjectExpr<'db> =
                rhs.check(check, env)
                    .await
                    .into_expr(check, env, &mut temporaries);

            // For now, let's do a dumb rule that operands must be
            // of the same primitive (and scalar) type.

            env.require_numeric_type(check, expr_span, lhs.ty(db));
            env.require_numeric_type(check, expr_span, rhs.ty(db));
            env.if_not_never(
                check,
                span_op.span,
                &[lhs.ty(db), rhs.ty(db)],
                async move |check, env| {
                    env.require_sub_object_type(&check, expr_span, lhs.ty(db), rhs.ty(db));
                    env.require_sub_object_type(&check, expr_span, rhs.ty(db), lhs.ty(db));
                },
            );

            // What type do we want these operators to have?
            // For now I'll just take the LHS, but that seems
            // wrong if e.g. one side is `!`, then we probably
            // want `!`, right?

            ExprResult::from_expr(
                check,
                env,
                ObjectExpr::new(
                    db,
                    expr_span,
                    lhs.ty(db),
                    ObjectExprKind::BinaryOp(span_op, lhs, rhs),
                ),
                temporaries,
            )
        }

        AstExprKind::Id(SpannedIdentifier { span: id_span, id }) => {
            match env.scope.resolve_name(db, *id, *id_span) {
                Err(reported) => ExprResult::err(db, reported),
                Ok(res) => ExprResult::from_name_resolution(check, env, res, expr_span),
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
                        // Got an error? Bail out.
                        Err(reported) => ExprResult::err(db, reported),

                        // Found something with lexical resolution? Continue.
                        Ok(Ok(r)) => ExprResult::from_name_resolution(check, env, r, expr_span),

                        // Otherwise, try type-dependent lookup.
                        Ok(Err(name_resolution)) => {
                            owner_result.kind = name_resolution.into();
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
            match owner_result.kind {
                ExprResultKind::Method {
                    self_expr: owner,
                    function: method,
                    generics: None,
                    id_span,
                } => {
                    let ast_terms = square_bracket_args.parse_as_generics(db);

                    ExprResult {
                        kind: ExprResultKind::Method {
                            self_expr: owner,
                            function: method,
                            generics: Some(ast_terms),
                            id_span,
                        },
                        ..owner_result
                    }
                }

                ExprResultKind::PlaceExpr(_) | ExprResultKind::Expr(_) => ExprResult::err(
                    db,
                    report_not_implemented(db, expr_span, "indexing expressions"),
                ),

                // We see something like `foo.bar[][]` where `bar` is a method.
                // The only correct thing here would be `foo.bar[]()[]`, i.e., call the method and then index.
                // We give an error under that assumption.
                // It seems likely we can do a better job.
                ExprResultKind::Method {
                    self_expr: owner,
                    function: method,
                    generics: Some(_),
                    ..
                } => ExprResult::err(db, report_missing_call_to_method(db, expr_span, method)),

                ExprResultKind::Other(name_resolution) => {
                    let generics = square_bracket_args.parse_as_generics(db);
                    match name_resolution.resolve_relative_generic_args(db, scope, &generics) {
                        Ok(name_resolution) => ExprResult {
                            temporaries: owner_result.temporaries,
                            span: expr_span,
                            kind: name_resolution.into(),
                        },
                        Err(r) => ExprResult::err(db, r),
                    }
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
                    check_method_call(
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

                ExprResult {
                    temporaries,
                    span: function_span,
                    kind:
                        ExprResultKind::Other(NameResolution {
                            generics,
                            sym: NameResolutionSym::SymFunction(sym),
                            ..
                        }),
                } => {
                    check_function_call(
                        check,
                        env,
                        function_span,
                        expr_span,
                        sym,
                        ast_args,
                        generics,
                        temporaries,
                    )
                    .await
                }

                // Calling a class like `Class(a, b)`: convert to
                // `Class.new(a, b)`.
                ExprResult {
                    temporaries,
                    span: class_span,
                    kind:
                        ExprResultKind::Other(
                            name_resolution @ NameResolution {
                                sym: NameResolutionSym::SymClass(class_sym),
                                ..
                            },
                        ),
                } => {
                    check_class_call(
                        check,
                        env,
                        class_span,
                        expr_span,
                        name_resolution,
                        class_sym,
                        ast_args,
                        temporaries,
                    )
                    .await
                }

                ExprResult {
                    span: owner_span, ..
                } => {
                    // FIXME: we probably want to support functions and function typed values?
                    ExprResult::err(db, report_not_callable(db, owner_span))
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
                ObjectExpr::new(
                    db,
                    expr_span,
                    ObjectTy::unit(db),
                    ObjectExprKind::Tuple(vec![]),
                )
            };

            let Some(expected_return_ty) = env.return_ty else {
                return ExprResult::err(
                    db,
                    Diagnostic::error(db, expr_span, format!("unexpected `return` statement"))
                        .label(
                            db,
                            Level::Error,
                            expr_span,
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
                span: expr_span,
                kind: ObjectExpr::new(
                    db,
                    expr_span,
                    ObjectTy::never(db),
                    ObjectExprKind::Return(return_expr),
                )
                .into(),
            }
        }

        AstExprKind::Await {
            future,
            await_keyword,
        } => {
            let future_span = future.span;
            let await_span = *await_keyword;

            let mut temporaries = vec![];

            let future_expr =
                future
                    .check(check, env)
                    .await
                    .into_expr(check, env, &mut temporaries);
            let future_ty = future_expr.ty(db);

            let awaited_ty = env.fresh_object_ty_inference_var(check);

            check.defer(env, await_span, async move |check, env| {
                let db = check.db;
                require_future(&check, &env, future_span, await_span, future_ty, awaited_ty).await
            });

            ExprResult {
                temporaries,
                span: expr_span,
                kind: ObjectExpr::new(
                    db,
                    expr_span,
                    awaited_ty,
                    ObjectExprKind::Await {
                        future: future_expr,
                        await_keyword: await_span,
                    },
                )
                .into(),
            }
        }
    }
}

async fn check_class_call<'db>(
    check: &Check<'db>,
    env: &Env<'db>,
    class_span: Span<'db>,
    expr_span: Span<'db>,
    name_resolution: NameResolution<'db>,
    class_sym: SymClass<'db>,
    ast_args: &SpanVec<'db, AstExpr<'db>>,
    temporaries: Vec<Temporary<'db>>,
) -> ExprResult<'db> {
    let db = check.db;

    let new_ident = SpannedIdentifier {
        span: class_span,
        id: Identifier::new_ident(db),
    };

    let (generics, new_function) = match name_resolution.resolve_relative_id(db, new_ident) {
        Ok(Ok(NameResolution {
            generics,
            sym: NameResolutionSym::SymFunction(m),
        })) => (generics, m),
        Ok(r) => return ExprResult::err(db, report_no_new_method(db, class_span, class_sym, r)),
        Err(reported) => return ExprResult::err(db, reported),
    };

    check_function_call(
        check,
        env,
        class_span,
        expr_span,
        new_function,
        ast_args,
        generics,
        temporaries,
    )
    .await
}

fn report_no_new_method<'db>(
    db: &'db dyn crate::Db,
    class_span: Span<'db>,
    class_sym: SymClass<'db>,
    resolution: Result<NameResolution<'db>, NameResolution<'db>>,
) -> Reported {
    let mut diag = Diagnostic::error(
        db,
        class_span,
        format!("the class `{class_sym}` has no `new` method"),
    )
    .label(
        db,
        Level::Error,
        class_span,
        format!("I could not find a `new` method on the class `{class_sym}`"),
    );

    match resolution {
        Ok(name_resolution) => {
            diag = diag.child(
                Diagnostic::new(
                    db,
                    Level::Note,
                    class_sym.name_span(db),
                    format!(
                        "calling a class is equivalent to calling `new`, but `new` is not a method"
                    ),
                )
                .label(
                    db,
                    Level::Note,
                    name_resolution.span(db).unwrap(),
                    format!(
                        "I found a class member named `new` but it is {}, not a method",
                        name_resolution.categorize(db)
                    ),
                ),
            );
        }
        Err(_) => {
            diag = diag.child(
                Diagnostic::new(
                    db,
                    Level::Note,
                    class_sym.name_span(db),
                    format!(
                        "calling a class is equivalent to calling `new`, but `{class_sym}` does not define a `new` method"
                    ),
                )
                .label(
                    db,
                    Level::Note,
                    class_sym.name_span(db),
                    format!("I could not find any class member named `new`"),
                ),
            );
        }
    }

    diag.report(db)
}

async fn require_future<'db>(
    check: &Check<'db>,
    env: &Env<'db>,
    future_span: Span<'db>,
    await_span: Span<'db>,
    future_ty: ObjectTy<'db>,
    awaited_ty: ObjectTy<'db>,
) {
    let db = check.db;

    let infer_var = match awaited_ty.kind(db) {
        ObjectTyKind::Infer(v) => *v,
        _ => unreachable!(),
    };

    let mut bounds = env.bounds(check, future_ty);

    while let Some(bound) = bounds.next().await {
        match bound {
            Bound::UpperBound(_) => continue,
            Bound::LowerBound(ty) => match ty.kind(db) {
                ObjectTyKind::Infer(_) => (),
                ObjectTyKind::Never => {
                    check.push_inference_var_bound(infer_var, Bound::LowerBound(ty.into()));
                }
                ObjectTyKind::Error(_) => {
                    check.push_inference_var_bound(infer_var, Bound::LowerBound(ty.into()));
                    return;
                }

                ObjectTyKind::Named(SymTyName::Future, vec) => {
                    let awaited_bound = vec[0].assert_type(db);
                    check.push_inference_var_bound(
                        infer_var,
                        Bound::LowerBound(awaited_bound.into()),
                    );
                }

                ObjectTyKind::Named(..) | ObjectTyKind::Var(..) => {
                    Diagnostic::error(db, await_span, format!("await requires a future"))
                        .label(
                            db,
                            Level::Error,
                            await_span,
                            format!("`await` requires a future"),
                        )
                        .label(db, Level::Info, future_span, format!("I found a {ty}"))
                        .report(db);
                    return;
                }
            },
        }
    }
}

async fn check_function_call<'db>(
    check: &Check<'db>,
    env: &Env<'db>,
    function_span: Span<'db>,
    expr_span: Span<'db>,
    function: SymFunction<'db>,
    ast_args: &SpanVec<'db, AstExpr<'db>>,
    generics: Vec<SymGenericTerm<'db>>,
    temporaries: Vec<Temporary<'db>>,
) -> ExprResult<'db> {
    let db = check.db;

    // Get the signature.
    let signature = function.signature(db);
    let input_output = signature.input_output(db);

    // Create inference vairables for any generic arguments not provided.
    let expected_generics = function.transitive_generic_parameters(db);
    let mut substitution = generics.clone();
    substitution.extend(
        expected_generics[generics.len()..]
            .iter()
            .map(|&var| env.fresh_inference_var(check, var.kind(db))),
    );

    check_call_common(
        check,
        env,
        function,
        expr_span,
        function_span,
        input_output,
        substitution,
        ast_args,
        None,
        temporaries,
    )
    .await
}

/// Check a call like `a.b()` where `b` is a method.
/// These are somewhat different than calls like `b(a)` because of how
/// type arguments are handled.
async fn check_method_call<'db>(
    check: &Check<'db>,
    env: &Env<'db>,
    id_span: Span<'db>,
    expr_span: Span<'db>,
    function: SymFunction<'db>,
    self_expr: Option<ObjectExpr<'db>>,
    ast_args: &[AstExpr<'db>],
    generics: Option<SpanVec<'db, AstGenericTerm<'db>>>,
    temporaries: Vec<Temporary<'db>>,
) -> ExprResult<'db> {
    let db = check.db;

    // Get the signature.
    let signature = function.signature(db);
    let input_output = signature.input_output(db);

    // Prepare the substitution for the function.
    let substitution = match generics {
        None => {
            // Easy case: nothing provided by user, just create inference variables for everything.
            env.existential_substitution(check, &input_output.variables)
        }

        Some(generics) => {
            // Harder case: user provided generics. Given that we are parsing a call like `a.b()`,
            // then generic arguments would be `a.b[x, y]()` and therefore correspond to the generics declared on
            // the function itself. But the `input_output` binder can contain additional parameters from
            // the class or surrounding scope. Those add'l parameters are instantiated with inference
            // variables-- and then the user-provided generics are added afterwards.

            // Create existential substitution for any other generics.
            let function_generics = &signature.symbols(db).generic_variables;
            assert!(input_output.variables.ends_with(function_generics));
            let outer_variables =
                &input_output.variables[0..input_output.variables.len() - function_generics.len()];
            let mut substitution: Vec<SymGenericTerm<'_>> =
                env.existential_substitution(check, outer_variables);

            // Check the user gave the expected number of arguments.
            if function_generics.len() != generics.len() {
                return ExprResult::err(
                    db,
                    Diagnostic::error(
                        db,
                        id_span,
                        format!(
                            "expected {expected} generic arguments, but found {found}",
                            expected = function_generics.len(),
                            found = generics.len()
                        ),
                    )
                    .label(
                        db,
                        Level::Error,
                        id_span,
                        format!(
                            "{found} generic arguments were provided",
                            found = generics.len()
                        ),
                    )
                    .label(
                        db,
                        Level::Error,
                        function.name_span(db),
                        format!(
                            "the function `{name}` is declared with {expected} generic arguments",
                            name = function.name(db),
                            expected = function_generics.len(),
                        ),
                    )
                    .report(db),
                );
            }

            // Convert each generic to a `SymGenericTerm` and check it has the correct kind.
            // If everything looks good, add it to the substitution.
            for (ast_generic_term, var) in generics.iter().zip(function_generics.iter()) {
                let sym_generic_term = ast_generic_term.into_sym_in_scope(db, &env.scope);
                if !sym_generic_term.has_kind(db, var.kind(db)) {
                    return ExprResult::err(
                        db,
                        Diagnostic::error(
                            db,
                            ast_generic_term.span(db),
                            format!(
                                "expected `{expected_kind}`, found `{found_kind}`",
                                expected_kind = var.kind(db),
                                found_kind = sym_generic_term.kind().unwrap(),
                            ),
                        )
                        .label(
                            db,
                            Level::Error,
                            id_span,
                            format!(
                                "this is a `{found_kind}`",
                                found_kind = sym_generic_term.kind().unwrap(),
                            ),
                        )
                        .label(
                            db,
                            Level::Info,
                            var.span(db),
                            format!(
                                "I expected to find a `{expected_kind}`",
                                expected_kind = var.kind(db),
                            ),
                        )
                        .report(db),
                    );
                }
                substitution.push(sym_generic_term);
            }

            substitution
        }
    };

    check_call_common(
        check,
        env,
        function,
        expr_span,
        id_span,
        input_output,
        substitution,
        ast_args,
        self_expr,
        temporaries,
    )
    .await
}

async fn check_call_common<'db>(
    check: &Check<'db>,
    env: &Env<'db>,
    function: SymFunction<'db>,
    expr_span: Span<'db>,
    callee_span: Span<'db>,
    input_output: &Binder<'db, Binder<'db, SymInputOutput<'db>>>,
    substitution: Vec<SymGenericTerm<'db>>,
    ast_args: &[AstExpr<'db>],
    self_expr: Option<ObjectExpr<'db>>,
    mut temporaries: Vec<Temporary<'db>>,
) -> ExprResult<'db> {
    let db = check.db;

    // Instantiate the input-output with the substitution.
    let input_output = input_output.substitute(db, &substitution);

    // Check the arity of the actual arguments.
    let expected_inputs = input_output.bound_value.input_tys.len();
    let found_inputs = ast_args.len();
    if ast_args.len() != expected_inputs {
        let function_name = function.name(db);
        return ExprResult::err(
            db,
            Diagnostic::error(
                db,
                callee_span,
                format!("expected {expected_inputs} arguments, found {found_inputs}"),
            )
            .label(
                db,
                Level::Error,
                callee_span,
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
    let arg_temp_span = |i: usize| ast_args.get(i).map(|a| a.span).unwrap_or(callee_span);
    let arg_temp_symbols = (0..expected_inputs)
        .map(|i| SymVariable::new(db, SymGenericKind::Place, None, arg_temp_span(i)))
        .collect::<Vec<_>>();
    let arg_temp_terms = arg_temp_symbols
        .iter()
        .map(|&sym| SymGenericTerm::var(db, sym))
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
            substitution,
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
            call_expr.ty(db),
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
