use dada_ir_ast::diagnostic::Errors;
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        inference::Direction,
        predicates::{Predicate, term_is_provably},
        report::{Because, OrElse, OrElseHelper},
    },
    ir::{
        indices::InferVarIndex,
        types::{SymGenericTerm, SymPerm},
        variables::SymVariable,
    },
};

use super::require_term_is;

pub fn test_var_is_provably<'db>(
    env: &mut Env<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
) -> bool {
    env.var_is_declared_to_be(var, predicate)
}

pub(super) fn require_var_is<'db>(
    env: &mut Env<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    if env.var_is_declared_to_be(var, predicate) {
        Ok(())
    } else {
        Err(or_else.report(env, Because::VarNotDeclaredToBe(var, predicate)))
    }
}

/// Requires the inference variable (appearing under the given context)
/// to meet the given predicate (possibly reporting an error
/// if that is contradictory).
///
/// If this is a type inference variable, we are ignoring the affiliated permission.
pub async fn require_infer_is<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    // The algorithms here are tied to the code in resolve.
    //
    // In particular to the fact that permissions fallback to `SymPerm::My`
    // if no bounds are found.

    let db = env.db();
    env.indent(
        "require_infer_is",
        &[&infer, &env.infer_var_kind(infer), &predicate],
        async |env| {
            // We leverage two key Lemmas.
            //
            // If `P1 <: P2` and for predicate C in {unique, owned, shared, lent} then
            //
            // 1. P2 is C => P1 is C
            // 2. not (P1 is C) => not (P2 is C)
            //
            // (Note that because of the possibility of `|` types,
            // Lemma 2 is weaker than it seems like it should be.
            // Intuitively I expected `P1 is shared => P2 is shared`,
            // for example, but because `our <: (our | my)`, that does not hold.
            // However, `not (P1 is unique) => not (P2 is unique)` does,
            // as does `not (P1 is shared) => not (Pr is shared)` etc.)
            //
            // Let's talk through the algorithm using the example of `Predicate::Owned`.
            // The Unique and Shared predicates are analogous.
            //
            // If we see a **lower bound** B1, we cannot not yet that the
            // final value will be owned. A future upper bound B2 could
            // be added that is not owned. However, Lemma 2 tells us that
            // B1 must be owned, since otherwise any future upper bound
            // could not be owned. So we can enforce that and then
            // continue waiting for future bounds.
            //
            // If see an **upper bound** B2, we can enforce it is owned
            // and then stop, since by Lemma 1, any future lower bound
            // B1 compatible with B2 must also be owned.
            //
            // If we reach the end of inference and we have only seen
            // lower bounds, then we can return successfully.
            // This is because the final value of the permission will be
            // equal to the final lower bound, and required it to be owned.
            // Otherwise we check that the fallback permission meets
            // the predicate.

            let mut bounds = env.term_bounds(perm, infer);
            let mut observed_lower_bound = false;
            while let Some((direction, bound)) = bounds.next(env).await {
                env.log("observed bound", &[&infer, &direction, &bound, &predicate]);

                require_term_is(env, bound, predicate, or_else).await?;

                match direction {
                    Direction::FromBelow => observed_lower_bound = true,
                    Direction::FromAbove => return Ok(()),
                }
            }

            env.log("observed_lower_bound", &[&observed_lower_bound]);

            if observed_lower_bound {
                Ok(())
            } else {
                require_term_is(
                    env,
                    perm.apply_to(db, SymGenericTerm::fallback(db, env.infer_var_kind(infer))),
                    predicate,
                    &or_else.map_because(move |_| Because::UnconstrainedInfer(infer)),
                )
                .await
            }
        },
    )
    .await
}

/// Wait until we know whether the inference variable IS the given predicate
/// or we know that we'll never be able to prove that it is.
#[boxed_async_fn]
pub async fn infer_is_provably<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
    infer: InferVarIndex,
    predicate: Predicate,
) -> Errors<bool> {
    match predicate {
        Predicate::Lent => {
            // If some lower bound is lent, then this must be lent.
            exists_bounding_term(
                env,
                perm,
                infer,
                Some(Direction::FromBelow),
                Predicate::Lent,
            )
            .await
        }

        Predicate::Owned => {
            // If some upper bound must be owned, the result must be owned.
            exists_bounding_term(
                env,
                perm,
                infer,
                Some(Direction::FromAbove),
                Predicate::Owned,
            )
            .await
        }

        Predicate::Shared | Predicate::Unique => {
            // If any bound is {shared, unique}, the result must be {shared, unique}.
            exists_bounding_term(env, perm, infer, None, predicate).await
        }
    }
}

async fn exists_bounding_term<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
    infer: InferVarIndex,
    direction: Option<Direction>,
    predicate: Predicate,
) -> Errors<bool> {
    env.indent("exists_bounding_term", &[&perm, &infer], async |env| {
        let db = env.db();
        let mut bounds = env.term_bounds(perm, infer);
        while let Some((direction_bound, bound)) = bounds.next(env).await {
            if let Some(direction) = direction
                && direction_bound != direction
            {
                continue;
            }

            if term_is_provably(env, perm.apply_to(db, bound), predicate).await? {
                return Ok(true);
            }
        }
        Ok(false)
    })
    .await
}
