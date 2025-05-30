use dada_ir_ast::{ast::AstStatement, span::Span};
use dada_util::boxed_async_fn;

use crate::{
    check::{CheckExprInEnv, env::Env, report::InvalidInitializerType},
    ir::{
        exprs::{SymExpr, SymExprKind},
        types::SymTy,
        variables::SymVariable,
    },
};

use super::{CheckTyInEnv, live_places::LivePlaces};

#[boxed_async_fn]
pub async fn check_block_statements<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    block_span: Span<'db>,
    statements: &[AstStatement<'db>],
) -> SymExpr<'db> {
    let db = env.db();

    let Some((first, rest)) = statements.split_first() else {
        return SymExpr::new(db, block_span, SymTy::unit(db), SymExprKind::Tuple(vec![]));
    };

    match first {
        AstStatement::Let(s) => {
            let lv = SymVariable::new_local(db, s.name(db).id, s.name(db).span);

            // For explicit local variables, we compute their type as a full symbol type first.
            let ty = match s.ty(db) {
                Some(ty) => ty.check_in_env(env).await,
                None => env.fresh_ty_inference_var(s.name(db).span),
            };

            let (initializer, body) = env
                .join(
                    async |env| match s.initializer(db) {
                        Some(initializer) => {
                            let initializer = initializer
                                .check_in_env(env, LivePlaces::fixme())
                                .await
                                .into_expr_with_enclosed_temporaries(env);
                            env.spawn_require_assignable_type(
                                LivePlaces::fixme(),
                                initializer.ty(db),
                                ty,
                                &InvalidInitializerType::new(lv, s.name(db).span, ty, initializer),
                            );
                            Some(initializer)
                        }

                        None => None,
                    },
                    async |env| {
                        env.push_program_variable_with_ty(lv, ty);
                        check_block_statements(env, LivePlaces::fixme(), block_span, rest).await
                    },
                )
                .await;

            // Create `let lv: ty = lv = initializer; remainder`
            let span = s.span(db).to(db, body.span(db));
            SymExpr::new(
                db,
                span,
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
            let check_e = async |env: &mut Env<'db>| {
                e.check_in_env(env, LivePlaces::fixme())
                    .await
                    .into_expr_with_enclosed_temporaries(env)
            };
            if rest.is_empty() {
                // Subtle-ish: if this is the last statement in the block,
                // it becomes the result of the block.
                check_e(env).await
            } else {
                let (ce, re) = env
                    .join(check_e, async |env| {
                        check_block_statements(env, live_after, block_span, rest).await
                    })
                    .await;
                SymExpr::new(
                    db,
                    ce.span(db).to(db, re.span(db)),
                    re.ty(db),
                    SymExprKind::Semi(ce, re),
                )
            }
        }
    }
}
