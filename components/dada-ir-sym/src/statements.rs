use std::future::Future;

use dada_ir_ast::{ast::AstStatement, span::Span};
use futures::join;

use crate::{
    env::Env,
    object_ir::{SymExpr, SymExprKind},
    symbol::SymVariable,
    ty::SymTy,
    CheckExprInEnv,
};

pub fn check_block_statements<'a, 'db>(
    env: &'a Env<'db>,
    block_span: Span<'db>,
    statements: &'a [AstStatement<'db>],
) -> impl Future<Output = SymExpr<'db>> + use<'a, 'db> {
    // (the box here permits recursion)
    Box::pin(async move {
        let db = env.db();

        let Some((first, rest)) = statements.split_first() else {
            return SymExpr::new(
                db,
                block_span,
                SymTy::unit(db),
                SymExprKind::Tuple(vec![]),
            );
        };

        match first {
            AstStatement::Let(s) => {
                let lv = SymVariable::new_local(db, s.name(db).id, s.name(db).span);

                // For explicit local variables, we compute their type as a full symbol type first.
                let ty = match s.ty(db) {
                    Some(ty) => env.symbolize(ty),
                    None => env.fresh_ty_inference_var(s.name(db).span),
                };

                let (initializer, body) = join!(
                    async {
                        match s.initializer(db) {
                            Some(initializer) => {
                                let initializer = initializer
                                    .check_expr_in_env(env)
                                    .await
                                    .into_expr_with_enclosed_temporaries(&env);
                                env.require_assignable_object_type(
                                    initializer.span(db),
                                    initializer.ty(db),
                                    ty,
                                );
                                Some(initializer)
                            }

                            None => None,
                        }
                    },
                    async {
                        let mut env = env.clone();
                        env.push_program_variable_with_ty(lv, ty);
                        check_block_statements(&env, block_span, rest).await
                    },
                );

                // Create `let lv: ty = lv = initializer; remainder`
                SymExpr::new(
                    db,
                    s.name(db).span,
                    body.ty(db),
                    SymExprKind::LetIn {
                        lv,
                        ty,
                        initializer,
                        body,
                    },
                )
            }

            AstStatement::Expr(e) => {
                let check_e = async {
                    e.check_expr_in_env(env)
                        .await
                        .into_expr_with_enclosed_temporaries(&env)
                };
                if rest.is_empty() {
                    // Subtle-ish: if this is the last statement in the block,
                    // it becomes the result of the block.
                    check_e.await
                } else {
                    let (ce, re) =
                        futures::join!(check_e, check_block_statements(env, block_span, rest));
                    SymExpr::new(
                        db,
                        ce.span(db).to(db, re.span(db)),
                        re.ty(db),
                        SymExprKind::Semi(ce, re),
                    )
                }
            }
        }
    })
}
