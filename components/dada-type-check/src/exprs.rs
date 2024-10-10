use dada_ir_ast::ast::{AstExpr, AstExprKind, BinaryOp};
use dada_ir_sym::ty::{SymTy, SymTyKind, SymTyName};

use crate::{
    checking_ir::{Expr, ExprKind},
    env::Env,
    executor::Check,
    Checking,
};

impl<'chk, 'db: 'chk> Checking<'chk, 'db> for AstExpr<'db> {
    type Checking = Expr<'chk, 'db>;

    fn check(&self, check: &mut Check<'chk, 'db>, env: &Env<'db>) -> Self::Checking {
        match &*self.kind {
            AstExprKind::Literal(literal) => {
                let ty = check.fresh_ty_var();
                check.defer_check(env, |check, env| todo!());
                check.expr(self.span, ty, ExprKind::Literal(literal.clone()))
            }

            AstExprKind::Tuple(span_vec) => {
                let exprs = span_vec
                    .values
                    .iter()
                    .map(|e| e.check(check, env))
                    .collect::<Vec<_>>();

                let ty = SymTy::new(
                    check.db,
                    SymTyKind::Named(
                        SymTyName::Tuple { arity: exprs.len() },
                        exprs.iter().map(|e| e.ty.into()).collect(),
                    ),
                );

                check.expr(self.span, ty, ExprKind::Tuple(exprs))
            }

            AstExprKind::BinaryOp(span_op, lhs, rhs) => {
                let lhs = lhs.check(check, env);
                let rhs = rhs.check(check, env);
                match span_op.op {
                    BinaryOp::Add => todo!(),
                    BinaryOp::Sub => todo!(),
                    BinaryOp::Mul => todo!(),
                    BinaryOp::Div => todo!(),
                }
            }

            AstExprKind::Id(spanned_identifier) => todo!(),
            AstExprKind::DotId(ast_expr, spanned_identifier) => todo!(),
            AstExprKind::SquareBracketOp(ast_expr, square_bracket_args) => todo!(),
            AstExprKind::ParenthesisOp(ast_expr, span_vec) => todo!(),
            AstExprKind::Constructor(ast_path, span_vec) => todo!(),
            AstExprKind::Return(ast_expr) => todo!(),
        }
    }
}
