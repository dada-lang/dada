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
    Checking,
};

struct ExprResult<'chk, 'db> {
    /// List of [`Temporary`][] variables created by this expression.
    temporaries: Vec<Temporary<'chk, 'db>>,

    /// Span of the expression
    span: Span<'db>,

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
struct Temporary<'chk, 'db> {
    lv: SymLocalVariable<'db>,
    expr: Expr<'chk, 'db>,
}

#[derive(FromImpls, Debug)]
enum ExprResultKind<'chk, 'db> {
    NameResolution(NameResolution<'db>),
    PlaceExpr(PlaceExpr<'chk, 'db>),
    Expr(Expr<'chk, 'db>),

    #[no_from_impl]
    Method {
        owner: Expr<'chk, 'db>,
        method: SymFunction<'db>,
        generics: Option<Vec<SymGenericTerm<'db>>>,
    },
}

impl<'chk, 'db: 'chk> Checking<'chk, 'db> for AstExpr<'db> {
    type Checking = ExprResult<'chk, 'db>;

    fn check(&self, check: &mut Check<'chk, 'db>, env: Env<'db>) -> Self::Checking {
        let db = check.db;

        match &*self.kind {
            AstExprKind::Literal(literal) => {
                let ty = env.fresh_ty_inference_var(check);
                check.defer_check(&env, |check, env| todo!());
                ExprResult {
                    temporaries: vec![],
                    span: self.span,
                    kind: check
                        .expr(self.span, ty, ExprKind::Literal(literal.clone()))
                        .into(),
                }
            }

            AstExprKind::Tuple(span_vec) => {
                let mut temporaries = vec![];
                let exprs = span_vec
                    .values
                    .iter()
                    .map(|e| e.check(check, env.clone()).to_expr(check, &mut temporaries))
                    .collect::<Vec<_>>();

                let ty = SymTy::new(
                    db,
                    SymTyKind::Named(
                        SymTyName::Tuple { arity: exprs.len() },
                        exprs.iter().map(|e| e.ty.into()).collect(),
                    ),
                );

                ExprResult {
                    temporaries,
                    span: self.span,
                    kind: ExprResultKind::Expr(check.expr(self.span, ty, ExprKind::Tuple(exprs))),
                }
            }

            AstExprKind::BinaryOp(span_op, lhs, rhs) => {
                let lhs = lhs.check(check, env.clone());
                let rhs = rhs.check(check, env);
                match span_op.op {
                    BinaryOp::Add => todo!(),
                    BinaryOp::Sub => todo!(),
                    BinaryOp::Mul => todo!(),
                    BinaryOp::Div => todo!(),
                }
            }

            AstExprKind::Id(SpannedIdentifier { span, id }) => {
                match env.scope.resolve_name(db, *id, *span) {
                    Err(r) => ExprResult::err(check, *span, r),
                    Ok(res) => ExprResult {
                        temporaries: vec![],
                        span: self.span,
                        kind: ExprResultKind::NameResolution(res),
                    },
                }
            }

            AstExprKind::DotId(owner, spanned_identifier) => {
                let owner_result = owner.check(check, env.clone());
                match owner_result.kind {}
            }
            AstExprKind::SquareBracketOp(ast_expr, square_bracket_args) => todo!(),
            AstExprKind::ParenthesisOp(ast_expr, span_vec) => todo!(),
            AstExprKind::Constructor(ast_path, span_vec) => todo!(),
            AstExprKind::Return(ast_expr) => todo!(),
        }
    }
}

impl<'chk, 'db> ExprResult<'chk, 'db> {
    fn err(check: &mut Check<'chk, 'db>, span: Span<'db>, r: Reported) -> Self {
        Self {
            temporaries: vec![],
            span,
            kind: ExprResultKind::Expr(check.err_expr(span, r)),
        }
    }

    fn to_place_expr(
        self,
        check: &mut Check<'chk, 'db>,
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

            ExprResultKind::NameResolution(name_resolution) => match name_resolution {
                NameResolution::SymLocalVariable(lv) => {
                    let ty = env.program_variable_ty(lv).unwrap();

                    check.place_expr(self.span, ty, PlaceExprKind::Local(lv))
                }

                NameResolution::SymFunction(_) // FIXME
                | NameResolution::SymModule(_)
                | NameResolution::SymClass(_)
                | NameResolution::SymPrimitive(_)
                | NameResolution::SymGeneric(..) => {
                    let r = Diagnostic::error(db, self.span, format!("expected a place expression"))
                    .label(db, Level::Error, self.span, format!("I expected to find a place in memory, like a local variable or field, but I found {}", name_resolution.categorize(db)))
                    .report(db);

                    check.place_expr(self.span, SymTy::error(db, r), PlaceExprKind::Error(r))
                }
            },

            ExprResultKind::Method {
                owner,
                method,
                generics,
            } => {
                let r = Diagnostic::error(db, self.span, format!("expected a place expression"))
                .label(db, Level::Error, self.span, format!("I expected to find a place in memory, like a local variable or field, but I found a method"))
                .report(db);

                check.place_expr(self.span, SymTy::error(db, r), PlaceExprKind::Error(r))
            }
        }
    }

    fn to_expr(
        self,
        check: &mut Check<'chk, 'db>,
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
            ExprResultKind::NameResolution(name_resolution) => todo!(),
            ExprResultKind::Method {
                owner,
                method,
                generics,
            } => todo!(),
        }
    }
}
