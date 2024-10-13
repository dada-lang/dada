use dada_ir_ast::ast::AstStatement;
use dada_ir_sym::{prelude::IntoSymInScope, symbol::SymLocalVariable};
use futures::join;

use crate::{
    checking_ir::{Expr, ExprKind},
    env::Env,
    executor::Check,
    Checking,
};

impl<'chk, 'db: 'chk> Checking<'chk, 'db> for [AstStatement<'db>] {
    type Checking = Expr<'chk, 'db>;

    async fn check(&self, check: &Check<'chk, 'db>, env: &Env<'db>) -> Self::Checking {
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

                let (initializer, body) = join!(
                    async {
                        match s.initializer(db) {
                            Some(initializer) => {
                                let initializer = initializer
                                    .check(check, env)
                                    .await
                                    .into_expr_with_enclosed_temporaries(check, &env);
                                env.require_subtype(check, initializer.ty, ty);
                                Some(initializer)
                            }

                            None => None,
                        }
                    },
                    async {
                        let mut env = env.clone();
                        env.insert_program_variable(lv, ty);
                        rest.check(check, &env).await
                    },
                );

                // Create `let lv: ty = lv = initializer; remainder`
                check.expr(
                    s.name(db).span,
                    body.ty,
                    ExprKind::LetIn {
                        lv,
                        ty,
                        initializer,
                        body,
                    },
                )
            }

            AstStatement::Expr(e) => {
                let check_e = async {
                    e.check(check, env)
                        .await
                        .into_expr_with_enclosed_temporaries(check, &env)
                };
                if rest.is_empty() {
                    check_e.await
                } else {
                    let (ce, re) = futures::join!(check_e, rest.check(check, env));
                    check.expr(ce.span.to(re.span), re.ty, ExprKind::Semi(ce, re))
                }
            }
        }
    }
}
