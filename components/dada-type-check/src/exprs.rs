use std::future::Future;

use dada_ir_ast::{
    ast::{AstExpr, AstExprKind, BinaryOp, Identifier, SpannedIdentifier},
    diagnostic::{Diagnostic, Level, Reported},
    span::Span,
};
use dada_ir_sym::{
    function::SymFunction,
    scope::NameResolution,
    symbol::SymLocalVariable,
    ty::{SymGenericTerm, SymTy, SymTyKind, SymTyName},
};
use dada_util::FromImpls;

use crate::{
    checking_ir::{Expr, ExprKind, PlaceExpr, PlaceExprKind},
    env::Env,
    executor::Check,
    member::lookup_member,
    Checking,
};

pub(crate) struct ExprResult<'chk, 'db> {
    /// List of [`Temporary`][] variables created by this expression.
    temporaries: Vec<Temporary<'chk, 'db>>,

    /// Span of the expression
    pub span: Span<'db>,

    /// The primary result from translating an expression.
    /// Note that an ast-expr can result in many kinds of things.
    kind: ExprResultKind<'chk, 'db>,
}

/// Translating an expression can result in the creation of
/// anonymous local temporaries that are injected into the
/// surrounding scope. These are returned alongside the result
/// and will eventually be translated into `let-in` expressions
/// when we reach the surrounding statement, block, or other
/// terminating context.
pub(crate) struct Temporary<'chk, 'db> {
    pub lv: SymLocalVariable<'db>,
    pub expr: Expr<'chk, 'db>,
}

#[derive(FromImpls, Debug)]
enum ExprResultKind<'chk, 'db> {
    /// An expression identifying a place in memory (e.g., a local variable).
    PlaceExpr(PlaceExpr<'chk, 'db>),

    /// An expression that produces a value.
    Expr(Expr<'chk, 'db>),

    /// A partially completed method call.
    #[no_from_impl]
    Method {
        owner: Expr<'chk, 'db>,
        method: SymFunction<'db>,
        generics: Option<Vec<SymGenericTerm<'db>>>,
    },

    /// Some kind of name resoluton that cannot be represented by as an expression.
    Other(NameResolution<'db>),
}

impl<'chk, 'db: 'chk> Checking<'chk, 'db> for AstExpr<'db> {
    type Checking = ExprResult<'chk, 'db>;

    fn check(
        &self,
        check: &Check<'chk, 'db>,
        env: &Env<'db>,
    ) -> impl Future<Output = Self::Checking> {
        Box::pin(check_expr(self, check, env))
    }
}

async fn check_expr<'chk, 'db>(
    expr: &AstExpr<'db>,
    check: &Check<'chk, 'db>,
    env: &Env<'db>,
) -> ExprResult<'chk, 'db> {
    let db = check.db;
    let span = expr.span;

    match &*expr.kind {
        AstExprKind::Literal(literal) => {
            let ty = env.fresh_ty_inference_var(check);
            check.defer(env, async move |check, env| todo!());
            ExprResult {
                temporaries: vec![],
                span,
                kind: check
                    .expr(span, ty, ExprKind::Literal(literal.clone()))
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
                        .into_expr(check, &mut temporaries),
                );
            }

            let ty = SymTy::new(
                db,
                SymTyKind::Named(
                    SymTyName::Tuple { arity: exprs.len() },
                    exprs.iter().map(|e| e.ty.into()).collect(),
                ),
            );

            ExprResult {
                temporaries,
                span,
                kind: ExprResultKind::Expr(check.expr(span, ty, ExprKind::Tuple(exprs))),
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
                Err(r) => ExprResult::err(check, *id_span, r),
                Ok(res) => ExprResult::from_name_resolution(check, env, res, span),
            }
        }

        AstExprKind::DotId(owner, id) => {
            let mut owner_result = owner.check(check, env).await;
            match owner_result.kind {
                ExprResultKind::PlaceExpr(_) | ExprResultKind::Expr(_) => {
                    lookup_member(check, &env, owner_result, *id).await
                }

                ExprResultKind::Other(name_resolution) => {
                    match name_resolution.resolve_relative_id(db, *id) {
                        Err(r) => ExprResult::err(check, span, r),
                        Ok(Ok(r)) => ExprResult::from_name_resolution(check, env, r, span),
                        Ok(Err(r)) => {
                            owner_result.kind = r.into();
                            lookup_member(check, &env, owner_result, *id).await
                        }
                    }
                }

                ExprResultKind::Method { owner, method, .. } => ExprResult::err(
                    check,
                    span,
                    report_missing_call_to_method(db, owner.span, method),
                ),
            }
        }
        AstExprKind::SquareBracketOp(ast_expr, square_bracket_args) => todo!(),
        AstExprKind::ParenthesisOp(ast_expr, span_vec) => todo!(),
        AstExprKind::Constructor(ast_path, span_vec) => todo!(),
        AstExprKind::Return(ast_expr) => todo!(),
    }
}

impl<'chk, 'db> ExprResult<'chk, 'db> {
    /// Create a result based on lexical name resolution.
    pub fn from_name_resolution(
        check: &Check<'chk, 'db>,
        env: &Env<'db>,
        res: NameResolution<'db>,
        span: Span<'db>,
    ) -> Self {
        match res {
            NameResolution::SymLocalVariable(lv) => {
                let ty = env.program_variable_ty(lv);
                let place_expr = check.place_expr(span, ty, PlaceExprKind::Local(lv));
                Self {
                    temporaries: vec![],
                    span,
                    kind: ExprResultKind::PlaceExpr(place_expr),
                }
            }

            // FIXME: Should functions be expressions?
            NameResolution::SymFunction(_)
            | NameResolution::SymModule(_)
            | NameResolution::SymClass(_)
            | NameResolution::SymPrimitive(_)
            | NameResolution::SymGeneric(..) => Self {
                temporaries: vec![],
                span,
                kind: ExprResultKind::Other(res),
            },
        }
    }

    /// Create an error result.
    pub fn err(check: &Check<'chk, 'db>, span: Span<'db>, r: Reported) -> Self {
        Self {
            temporaries: vec![],
            span,
            kind: ExprResultKind::Expr(check.err_expr(span, r)),
        }
    }

    /// Convert this result into an expression, with `let ... in` statements inserted for temporaries.
    pub fn into_expr_with_enclosed_temporaries(
        self,
        check: &Check<'chk, 'db>,
        env: &Env<'db>,
    ) -> Expr<'chk, 'db> {
        let mut temporaries = vec![];
        let mut expr = self.into_expr(check, &mut temporaries);
        for temporary in temporaries.into_iter().rev() {
            expr = check.expr(
                expr.span,
                expr.ty,
                ExprKind::LetIn {
                    lv: temporary.lv,
                    ty: temporary.expr.ty,
                    initializer: Some(temporary.expr),
                    body: expr,
                },
            );
        }
        expr
    }

    /// Computes the type of this, treating it as an expression.
    /// Reports an error if this names something that cannot be made into an expression.
    pub fn ty(self, check: &Check<'chk, 'db>, env: &Env<'db>) -> SymTy<'db> {
        let db = check.db;
        match self.kind {
            ExprResultKind::PlaceExpr(place_expr) => place_expr.ty,
            ExprResultKind::Expr(expr) => expr.ty,
            ExprResultKind::Other(name_resolution) => {
                SymTy::error(db, report_non_expr(db, self.span, name_resolution))
            }
            ExprResultKind::Method { owner, method, .. } => {
                SymTy::error(db, report_missing_call_to_method(db, owner.span, method))
            }
        }
    }

    pub fn into_place_expr(
        self,
        check: &Check<'chk, 'db>,
        env: &Env<'db>,
        temporaries: &mut Vec<Temporary<'chk, 'db>>,
    ) -> PlaceExpr<'chk, 'db> {
        let db = check.db;
        temporaries.extend(self.temporaries);
        match self.kind {
            ExprResultKind::PlaceExpr(place_expr) => place_expr,

            // This is a value that needs to be stored in a temporary.
            ExprResultKind::Expr(expr) => {
                let ty = expr.ty;

                // Create a temporary to store the result of this expression.
                let name = Identifier::new(db, format!("#tmp{expr:?}"));
                let lv = SymLocalVariable::new(db, name, expr.span);
                temporaries.push(Temporary { lv, expr: expr });

                // The result will be a reference to that temporary.
                check.place_expr(self.span, ty, PlaceExprKind::Local(lv))
            }

            ExprResultKind::Other(name_resolution) => {
                let r = report_non_expr(db, self.span, name_resolution);
                check.place_expr(self.span, SymTy::error(db, r), PlaceExprKind::Error(r))
            }

            ExprResultKind::Method { owner, method, .. } => {
                let r = report_missing_call_to_method(db, owner.span, method);
                check.place_expr(self.span, SymTy::error(db, r), PlaceExprKind::Error(r))
            }
        }
    }

    pub fn into_expr(
        self,
        check: &Check<'chk, 'db>,
        temporaries: &mut Vec<Temporary<'chk, 'db>>,
    ) -> Expr<'chk, 'db> {
        let db = check.db;
        temporaries.extend(self.temporaries);
        match self.kind {
            ExprResultKind::Expr(expr) => expr,
            ExprResultKind::PlaceExpr(place_expr) => check.expr(
                place_expr.span,
                place_expr.ty.shared(db, place_expr.to_sym_place(db)),
                ExprKind::Share(place_expr),
            ),
            ExprResultKind::Other(name_resolution) => {
                check.err_expr(self.span, report_non_expr(db, self.span, name_resolution))
            }
            ExprResultKind::Method { owner, method, .. } => check.err_expr(
                self.span,
                report_missing_call_to_method(db, self.span, method),
            ),
        }
    }
}

fn report_non_expr<'db>(
    db: &'db dyn crate::Db,
    owner_span: Span<'db>,
    name_resolution: NameResolution<'db>,
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
