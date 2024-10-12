use dada_ir_ast::ast::{AstBlock, AstStatement};
use dada_ir_sym::{
    function::{SymFunction, SymInputOutput},
    prelude::IntoSymInScope,
    symbol::SymLocalVariable,
    ty::SymTy,
};

use crate::{
    checking_ir::{Expr, ExprKind, PlaceExprKind},
    env::Env,
    executor::Check,
    Checking,
};

impl<'chk, 'db: 'chk> Checking<'chk, 'db> for [AstStatement<'db>] {
    type Checking = Expr<'chk, 'db>;

    fn check(&self, check: &mut Check<'chk, 'db>, mut env: Env<'db>) -> Self::Checking {
        let db = check.db;

        let Some((first, rest)) = self.split_first() else {
            panic!("empty list of statements")
        };

        match first {
            AstStatement::Let(s) => {
                let lv = SymLocalVariable::new(db, s.name(db).id, s.name(db).span);

                let ty = match s.ty(db) {
                    Some(ty) => ty.into_sym_in_scope(db, &env.scope),
                    None => env.fresh_ty_inference_var(check),
                };

                let assign_initializer = match s.initializer(db) {
                    Some(initializer) => {
                        let initializer = initializer.check(check, env.clone());
                        env.require_subtype(check, initializer.ty, ty);
                        let lv_place =
                            check.place_expr(lv.name_span(db), ty, PlaceExprKind::Local(lv));
                        Some(check.expr(
                            initializer.span,
                            check.unit(),
                            ExprKind::Assign(lv_place, initializer),
                        ))
                    }
                    None => None,
                };

                env.insert_program_variable(lv, ty);
                let remainder = rest.check(check, env);

                // Create `let lv: ty = lv = initializer; remainder`
                let body = check
                    .exprs(assign_initializer.into_iter().chain(Some(remainder)))
                    .unwrap();
                check.expr(s.name(db).span, body.ty, ExprKind::LetIn(lv, ty, body))
            }

            AstStatement::Expr(e) => {
                let ce = e.check(check, env.clone());
                if rest.is_empty() {
                    ce
                } else {
                    let re = rest.check(check, env);
                    check.expr(ce.span.to(re.span), re.ty, ExprKind::Semi(ce, re))
                }
            }
        }
    }
}
