use dada_ir_ast::diagnostic::Errors;
use dada_util::boxed_async_fn;

use crate::{
    check::{
        env::Env,
        inference::Direction,
        predicates::{Predicate, term_is_provably},
        report::{Because, OrElse},
    },
    ir::{indices::InferVarIndex, types::SymPerm, variables::SymVariable},
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
    if env.infer_is(infer, predicate).is_some() {
        // Already required to meet this predicate.
        return Ok(());
    }

    if let Some(_or_else_invert) = env.infer_is(infer, predicate.invert()) {
        // Already required NOT to meet this predicate.
        return Err(or_else.report(env, Because::JustSo)); // FIXME we can do better than JustSo
    }

    // Record that `infer` is required to meet the predicate.
    env.set_infer_is(infer, predicate, or_else);

    // Enforce the result implications of that
    match predicate {
        Predicate::Lent => {
            // If `infer` must be lent, its upper bounds must be lent.
            // Lower bounds don't have to be.
            //
            // FIXME: Well, lower bounds have to be "maybe" lent.
            require_bounding_terms_are(
                env,
                perm,
                infer,
                Some(Direction::FromAbove),
                Predicate::Lent,
                or_else,
            )
            .await
        }

        Predicate::Owned => {
            // If `infer` must be owned, its lower bounds must be owned.
            // Upper bounds don't have to be.
            //
            // FIXME: Well, upper bounds have to be "maybe" owned.
            require_bounding_terms_are(
                env,
                perm,
                infer,
                Some(Direction::FromBelow),
                Predicate::Owned,
                or_else,
            )
            .await
        }

        Predicate::Shared | Predicate::Unique => {
            require_bounding_terms_are(env, perm, infer, None, predicate, or_else).await
        }
    }
}

async fn require_bounding_terms_are<'db>(
    env: &mut Env<'db>,
    perm: SymPerm<'db>,
    infer: InferVarIndex,
    direction: Option<Direction>,
    predicate: Predicate,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    env.indent(
        "require_bounding_terms_are",
        &[&perm, &infer],
        async |env| {
            let mut bounds = env.term_bounds(perm, infer, direction);
            while let Some((_, bound)) = bounds.next(env).await {
                require_term_is(env, bound, predicate, or_else).await?;
            }
            Ok(())
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
    if env.infer_is(infer, predicate).is_some() {
        // Already required to meet this predicate.
        return Ok(true);
    }

    if let Some(_or_else_invert) = env.infer_is(infer, predicate.invert()) {
        // Already required NOT to meet this predicate.
        return Ok(false);
    }

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
        let mut bounds = env.term_bounds(perm, infer, direction);
        while let Some((_, bound)) = bounds.next(env).await {
            if term_is_provably(env, perm.apply_to(db, bound), predicate).await? {
                return Ok(true);
            }
        }
        Ok(false)
    })
    .await
}
