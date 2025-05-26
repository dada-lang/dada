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

use super::{is_provably_lent::term_is_provably_lent, require_term_is};

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
        async |env| match predicate {
            Predicate::Owned | Predicate::Unique | Predicate::Shared => {
                // We leverage two key Lemmas in these 3 cases.
                // `Predicate::Lent` is handled separately, as
                // Lemma 1 does not hold (e.g., `our <: ref[_]`).
                //
                // If `P1 <: P2` then
                //
                // 1. (P2 is unique => P1 is unique, P2 is owned => P1 is owned, P2 is shared => P1 is shared).
                // 2. not (P1 is unique) => not (P2 is unique), not (P1 is owned) => not (P2 is owned), not (P1 is shared) => not (P2 is shared)
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
            }

            Predicate::Lent => {
                // Lent predicates are trickier. The challenge is that
                // the algorithm for selecting the final value of an
                // inference variable prefers the lower bound over the
                // upper bound. This normally yields a better overall
                // value as lower bounds represent incoming sources of
                // types and are generally more precise. *However*,
                // if a *lent* predicate is required, using lower bounds
                // poses a challenge, as we might have a lower bound of
                // `our` and an upper bound like `ref[x]`.
                //
                // We observe the follow Lemmas. If `P1 <: P2` then
                //
                // 1. not (P2 is lent) => not (P1 is lent)

                let mut bounds = env.term_bounds(perm, infer);
                let mut observed_lower_bound = false;
                let mut observed_upper_bound = false;
                let mut lower_bound_is_lent = false;
                while let Some((direction, bound)) = bounds.next(env).await {
                    match direction {
                        Direction::FromBelow => {
                            // Lower bounds do not necessarily have to be lent.
                            // e.g., you could have `our <: ?X <: ref[x]`.
                            observed_lower_bound = true;
                            lower_bound_is_lent = term_is_provably_lent(env, bound).await?;
                        }
                        Direction::FromAbove => {
                            // Per Lemma 2, upper bounds must be lent. However, once we have proved this,
                            // we cannot necessarily stop iterating.
                            require_term_is(env, bound, predicate, or_else).await?;
                            observed_upper_bound = true;
                        }
                    }
                }
                env.log("observed_lower_bound", &[&observed_lower_bound]);
                env.log("observed_upper_bound", &[&observed_upper_bound]);
                env.log("lower_bound_is_lent", &[&lower_bound_is_lent]);
                if lower_bound_is_lent || (!observed_lower_bound && observed_upper_bound) {
                    Ok(())
                } else {
                    Err(or_else.report(env, Because::UnconstrainedInfer(infer)))
                }
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
            if let Some(direction) = direction {
                if direction_bound != direction {
                    continue;
                }
            }

            if term_is_provably(env, perm.apply_to(db, bound), predicate).await? {
                return Ok(true);
            }
        }
        Ok(false)
    })
    .await
}
