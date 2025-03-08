use dada_ir_ast::diagnostic::Errors;
use dada_util::{boxed_async_fn, vecset::VecSet};

use crate::{
    check::{
        combinator::{self, exists, require, require_for_all, require_for_all_infer_bounds},
        env::Env,
        inference::InferenceVarData,
        predicates::{
            Predicate, is_provably_copy::term_is_provably_copy, require_copy::require_term_is_copy,
            require_term_is_my, term_is_provably_my,
        },
        report::{Because, OrElse},
    },
    ir::{
        indices::InferVarIndex,
        red::{Chain, Lien},
    },
};

use super::alternatives::Alternative;

// Rules (ignoring inference and layout rules)
//
// * `my <= C`
// * `our <= C1 if C1 is copy`
// * `(our C0) <= (our C1) if C0 <= C1`
// * `(leased[place0] C0) <= (leased[place1] C1) if place1 <= place0 && C0 <= C1`
// * `(shared[place0] C0) <= (shared[place1] C1) if place1 <= place0 && C0 <= C1`
// * `(shared[place0] C0) <= (our C1) if (leased[place0] C0) <= C1`
// * `X C0 <= X C1 if C0 <= C1`
// * `X <= our if X is copy+owned`
// * `X <= my if X is move+owned`

pub async fn require_sub_red_perms<'a, 'db>(
    env: &'a Env<'db>,
    lower_chains: &VecSet<Chain<'db>>,
    upper_chains: &VecSet<Chain<'db>>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    require_for_all(lower_chains, async |lower_chain| {
        require_sub_some(env, lower_chain, upper_chains, or_else).await
    })
    .await
}

async fn require_sub_some<'a, 'db>(
    env: &'a Env<'db>,
    lower_chain: &Chain<'db>,
    upper_chains: &VecSet<Chain<'db>>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let mut root = Alternative::root();
    let children_alternatives = root.spawn_children(upper_chains.len());
    require(
        exists(
            upper_chains.into_iter().zip(children_alternatives),
            async |(upper_chain, mut child_alternative)| {
                sub_chains(
                    env,
                    &mut child_alternative,
                    &lower_chain,
                    &upper_chain,
                    or_else,
                )
                .await
            },
        ),
        || or_else.report(env, Because::JustSo),
    )
    .await
}

#[boxed_async_fn]
async fn sub_chains<'a, 'db>(
    env: &'a Env<'db>,
    alternative: &mut Alternative<'_>,
    lower_chain: &[Lien<'db>],
    upper_chain: &[Lien<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<bool> {
    let db = env.db();
    match (lower_chain.split_first(), upper_chain.split_first()) {
        (None, _) => {
            // `my <= C`
            Ok(true)
        }

        (Some(_), None) => {
            let lower_term = Lien::chain_to_perm(db, lower_chain);
            alternative
                .if_required(
                    require_term_is_my(env, lower_term.into(), or_else),
                    term_is_provably_my(env, lower_term.into()),
                )
                .await
        }

        (Some((&lien0, c0)), Some((&lien1, c1))) => {
            sub_chains1(env, alternative, lien0, c0, lien1, c1, or_else).await
        }
    }
}

#[boxed_async_fn]
async fn sub_chains1<'a, 'db>(
    env: &'a Env<'db>,
    alternative: &mut Alternative<'_>,
    lower_chain_head: Lien<'db>,
    lower_chain_tail: &[Lien<'db>],
    upper_chain_head: Lien<'db>,
    upper_chain_tail: &[Lien<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<bool> {
    let db = env.db();
    match (
        lower_chain_head,
        lower_chain_tail,
        upper_chain_head,
        upper_chain_tail,
    ) {
        (Lien::Error(reported), _, _, _) | (_, _, Lien::Error(reported), _) => Err(reported),

        (Lien::Infer(v0), c0, _, _) => {
            if c0.is_empty() {
                alternative
                    .if_required(
                        require_upper_chain(env, v0, upper_chain_head, upper_chain_tail, or_else),
                        splice_lower_bound(
                            env,
                            v0,
                            c0,
                            upper_chain_head,
                            upper_chain_tail,
                            or_else,
                        ),
                    )
                    .await
            } else {
                splice_lower_bound(env, v0, c0, upper_chain_head, upper_chain_tail, or_else).await
            }
        }

        (_, _, Lien::Infer(v1), c1) => {
            if c1.is_empty() {
                alternative
                    .if_required(
                        require_lower_chain(env, upper_chain_head, upper_chain_tail, v1, or_else),
                        splice_upper_bound(
                            env,
                            lower_chain_head,
                            lower_chain_tail,
                            v1,
                            c1,
                            or_else,
                        ),
                    )
                    .await
            } else {
                splice_upper_bound(env, lower_chain_head, lower_chain_tail, v1, c1, or_else).await
            }
        }

        (Lien::Our, [], head1, tail1) => {
            // `our <= C1 if C1 is copy`
            let perm1 = Lien::head_tail_to_perm(db, head1, tail1);
            alternative
                .if_required(
                    require_term_is_copy(env, perm1.into(), or_else),
                    term_is_provably_copy(env, perm1.into()),
                )
                .await
        }
        (Lien::Our, c0, Lien::Our, c1) => {
            // `(our C0) <= (our C1) if C0 <= C1`
            sub_chains(env, alternative, c0, c1, or_else).await
        }
        (Lien::Our, _, Lien::Leased(_), _) => Ok(false),
        (Lien::Our, _, Lien::Shared(_), _) => Ok(false),
        (Lien::Our, _, Lien::Var(_), _) => Ok(false),

        (Lien::Leased(_), _, Lien::Our, _) => Ok(false),
        (Lien::Leased(place0), c0, Lien::Leased(place1), c1) => {
            // * `(leased[place0] C0) <= (leased[place1] C1) if place1 <= place0 && C0 <= C1`
            if place0.is_covered_by(db, place1) {
                sub_chains(env, alternative, c0, c1, or_else).await
            } else {
                Ok(false)
            }
        }
        (Lien::Leased(_), _, Lien::Shared(_), _) => Ok(false),
        (Lien::Leased(_), _, Lien::Var(_), _) => Ok(false),

        (Lien::Shared(place0), c0, Lien::Our, [lien1, c1 @ ..]) => {
            // * `(shared[place0] C0) <= (our C1) if (leased[place0] C0) <= C1`
            sub_chains1(
                env,
                alternative,
                Lien::Leased(place0),
                c0,
                *lien1,
                c1,
                or_else,
            )
            .await
        }
        (Lien::Shared(_), _, Lien::Our, []) => {
            // See above rule: if C1 is [] then `leased[place0] C0 <= []` will also be false.
            Ok(false)
        }
        (Lien::Shared(place0), c0, Lien::Shared(place1), c1) => {
            // * `(shared[place0] C0) <= (shared[place1] C1) if place1 <= place0 && C0 <= C1`
            if place0.is_covered_by(db, place1) {
                sub_chains(env, alternative, c0, c1, or_else).await
            } else {
                Ok(false)
            }
        }
        (Lien::Shared(_), _, Lien::Leased(_), _) => Ok(false),
        (Lien::Shared(_), _, Lien::Var(_), _) => Ok(false),

        (Lien::Var(v0), [], Lien::Our, []) => {
            // `X <= our`
            Ok(env.var_is_declared_to_be(v0, Predicate::Copy)
                && env.var_is_declared_to_be(v0, Predicate::Owned))
        }
        (Lien::Var(_), _, Lien::Our, _) => Ok(false),
        (Lien::Var(v0), c0, Lien::Var(v1), c1) => {
            // * `X C0 <= X C1 if C0 <= C1`
            if v0 == v1 {
                sub_chains(env, alternative, c0, c1, or_else).await
            } else {
                Ok(false)
            }
        }
        (Lien::Var(_), _, Lien::Leased(_), _) => Ok(false),
        (Lien::Var(_), _, Lien::Shared(_), _) => Ok(false),
    }
}

/// Covers the case where `L0 Ln <= ?U0`. This adds an upper bounding chain
/// to `?L0`.
async fn require_lower_chain<'db>(
    env: &Env<'db>,
    lower_head: Lien<'db>,
    lower_tail: &[Lien<'db>],
    upper_head: InferVarIndex,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let lower_chain = Chain::from_head_tail(env.db(), lower_head, lower_tail);

    let Some(or_else) = env
        .runtime()
        .insert_lower_chain(upper_head, &lower_chain, or_else)
    else {
        return Ok(());
    };

    // If this is a new lower bound, spawn a task that will check that
    // there is at least one *upper bound* on the variable (either one
    // that currently exists or one that may be added in the future)
    // that is a superchain of this lower bound.
    env.runtime().spawn(env, async move |ref env| {
        require_for_all_infer_bounds(
            env,
            upper_head,
            InferenceVarData::upper_chains,
            async |upper_chain| {
                Alternative::the_future_never_comes(async |alternative| {
                    require(
                        sub_chains(env, alternative, &lower_chain, &upper_chain, &or_else),
                        || or_else.report(env, Because::JustSo),
                    )
                    .await
                })
                .await
            },
        )
        .await
    });

    Ok(())
}

/// This is the general case routine for handling the scenario:
///
/// ```
/// L0 Ln <= ?U0 Un
/// ```
///
/// where `L0` is the first lien in the lower chain, `Ln` are the remaining liens,
/// and the first lien `?U0` of the upper chain is an inference variable,
/// followed by the remaining liens `Un`.
///
/// It works by "splicing" any upper bounds `B` of `?U0` in front of `Un`
/// and searching for a case where the `L0 Ln <= B Un`. It's enough to find
/// *any* upper bound because the final permission must be smaller
/// than all of them.
///
/// It's a bit counterintuive that we splice the UPPER bounds of `?U0`.
/// But think about it, if we have a lower bound `LB <= ?U0`, there is
/// no necessary relation between that and `L0 Ln` (*). But if we have an
/// upper bound `?0 <= UB`, then by transitivity `L0 Ln <= UB` must hold.
///
/// (*) This is true but we ought to be propagating "layout".
async fn splice_upper_bound<'db>(
    env: &Env<'db>,
    lower_head: Lien<'db>,
    lower_tail: &[Lien<'db>],
    upper_head: InferVarIndex,
    upper_tail: &[Lien<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<bool> {
    let lower_chain = Chain::from_head_tail(env.db(), lower_head, lower_tail);
    combinator::exists_infer_bound(
        env,
        upper_head,
        InferenceVarData::upper_chains,
        async |mut upper_chain| {
            Alternative::the_future_never_comes(async |alternative| {
                upper_chain.extend(upper_tail);
                sub_chains(env, alternative, &lower_chain, &upper_chain, or_else).await
            })
            .await
        },
    )
    .await
}

/// Covers the case where `?L0 <= U0 Un`. This adds an upper bounding chain
/// to `?L0`.
async fn require_upper_chain<'db>(
    env: &Env<'db>,
    lower_head: InferVarIndex,
    upper_head: Lien<'db>,
    upper_tail: &[Lien<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let upper_chain = Chain::from_head_tail(env.db(), upper_head, upper_tail);

    let Some(_or_else) = env
        .runtime()
        .insert_upper_chain(lower_head, &upper_chain, or_else)
    else {
        return Ok(());
    };

    // Interesting observation: We don't actually need to check for
    // consistency with lower-bounds here. If there are any lower-bounds,
    // they will have spawned a task that is checking upper chains.

    Ok(())
}

/// Like [`splice_upper_bound`][] but covers the case `?L0 Ln <= U0 Un`.
async fn splice_lower_bound<'db>(
    env: &Env<'db>,
    lower_head: InferVarIndex,
    lower_tail: &[Lien<'db>],
    upper_head: Lien<'db>,
    upper_tail: &[Lien<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<bool> {
    let upper_chain = Chain::from_head_tail(env.db(), upper_head, upper_tail);
    combinator::exists_infer_bound(
        env,
        lower_head,
        InferenceVarData::lower_chains,
        async |mut lower_chain| {
            Alternative::the_future_never_comes(async |alternative| {
                lower_chain.extend(lower_tail);
                sub_chains(env, alternative, &lower_chain, &upper_chain, or_else).await
            })
            .await
        },
    )
    .await
}
