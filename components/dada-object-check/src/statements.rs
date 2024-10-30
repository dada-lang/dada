use std::future::Future;

use dada_ir_ast::{ast::AstStatement, span::Span};
use dada_ir_sym::{prelude::IntoSymInScope, symbol::SymVariable};
use futures::join;

use crate::{
    env::Env,
    object_ir::{IntoObjectIr, ObjectExpr, ObjectExprKind, ObjectTy},
    Checking,
};

pub fn check_block_statements<'a, 'db>(
    env: &'a Env<'db>,
    block_span: Span<'db>,
    statements: &'a [AstStatement<'db>],
) -> impl Future<Output = ObjectExpr<'db>> + use<'a, 'db> {
    // (the box here permits recursion)
    Box::pin(async move {
        let db = env.db();

        let Some((first, rest)) = statements.split_first() else {
            return ObjectExpr::new(
                db,
                block_span,
                ObjectTy::unit(db),
                ObjectExprKind::Tuple(vec![]),
            );
        };

        match first {
            AstStatement::Let(s) => {
                let lv = SymVariable::new_local(db, s.name(db).id, s.name(db).span);

                // For explicit local variables, we compute their type as a full symbol type first.
                let ty = match s.ty(db) {
                    Some(ty) => ty.into_sym_in_scope(db, &env.scope),
                    None => env.fresh_ty_inference_var(s.name(db).span),
                };

                let (initializer, body) = join!(
                    async {
                        match s.initializer(db) {
                            Some(initializer) => {
                                let initializer = initializer
                                    .check(env)
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
                        env.insert_program_variable(lv, ty);
                        check_block_statements(&env, block_span, rest).await
                    },
                );

                // Create `let lv: ty = lv = initializer; remainder`
                ObjectExpr::new(
                    db,
                    s.name(db).span,
                    body.ty(db),
                    ObjectExprKind::LetIn {
                        lv,
                        sym_ty: Some(ty),
                        ty: ty.into_object_ir(db),
                        initializer,
                        body,
                    },
                )
            }

            AstStatement::Expr(e) => {
                let check_e =
                    async { e.check(env).await.into_expr_with_enclosed_temporaries(&env) };
                if rest.is_empty() {
                    // Subtle-ish: if this is the last statement in the block,
                    // it becomes the result of the block.
                    check_e.await
                } else {
                    let (ce, re) =
                        futures::join!(check_e, check_block_statements(env, block_span, rest));
                    ObjectExpr::new(
                        db,
                        ce.span(db).to(re.span(db)),
                        re.ty(db),
                        ObjectExprKind::Semi(ce, re),
                    )
                }
            }
        }
    })
}
