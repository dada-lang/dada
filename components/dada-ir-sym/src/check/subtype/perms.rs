use dada_ir_ast::diagnostic::Errors;
use dada_util::{boxed_async_fn, vecset::VecSet};

use crate::{
    check::{
        alternatives::Alternative,
        debug::TaskDescription,
        env::Env,
        inference::Direction,
        live_places::LivePlaces,
        predicates::{
            Predicate,
            is_provably_copy::{perm_is_provably_copy, term_is_provably_copy},
            is_provably_move::perm_is_provably_move,
            is_provably_owned::perm_is_provably_owned,
            require_copy::require_term_is_copy,
            require_term_is_my, term_is_provably_my,
        },
        red::{RedPerm, RedPermLink},
        report::{Because, OrElse},
        to_red::ToRedPerm,
    },
    ir::{indices::InferVarIndex, types::SymPerm},
};

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

pub async fn require_sub_opt_perms<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    lower_perm: Option<SymPerm<'db>>,
    upper_perm: Option<SymPerm<'db>>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();
    let lower_perm = lower_perm.unwrap_or_else(|| SymPerm::my(db));
    let upper_perm = upper_perm.unwrap_or_else(|| SymPerm::my(db));
    require_sub_perms(env, live_after, lower_perm, upper_perm, or_else).await
}

async fn require_sub_perms<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    lower_perm: SymPerm<'db>,
    upper_perm: SymPerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    // First check if this is a case of `my <: P` (true for any P where P != leased)
    // or `our <: P` (true for any P where P is copy).
    if perm_is_provably_owned(env, lower_perm).await? {
        if perm_is_provably_move(env, lower_perm).await? {
            return require_sub_my(env, upper_perm, or_else).await;
        } else if perm_is_provably_copy(env, lower_perm).await? {
            return require_sub_our(env, upper_perm, or_else).await;
        }
    }

    simplify_red_perm(
        env,
        live_after,
        Direction::FromAbove,
        lower_perm.to_red_perm(env),
        async |env, lower_red_perm| {
            simplify_red_perm(
                env,
                live_after,
                Direction::FromBelow,
                upper_perm.to_red_perm(env),
                async |env, upper_red_perm| {},
            )
            .await
        },
    )
    .await
}


async fn require_sub_my<'db>(
    env: &mut Env<'db>,
    upper_perm: SymPerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    // `our <: P` if `P` is copy
    //
    // NB. Simplification cannot change whether `P` is owned or copy.
    env.require(
        async |env| {
            env.either(
                async |env| perm_is_provably_owned(env, upper_perm).await,
                async |env| perm_is_provably_copy(env, upper_perm).await,
            )
            .await
        },
        |env| or_else.report(env, Because::JustSo),
    )
    .await
}

async fn require_sub_our<'db>(
    env: &mut Env<'db>,
    upper_perm: SymPerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    // `our <: P` if `P` is copy
    //
    // NB. Simplification cannot change whether `P` is copy.
    require_term_is_copy(env, upper_perm.into(), or_else).await
}

async fn require_sub_some<'db>(
    env: &mut Env<'db>,
    lower_chain: &RedPerm<'db>,
    upper_chains: &VecSet<RedPerm<'db>>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let mut root = Alternative::root();
    let children_alternatives = root.spawn_children(upper_chains.len());
    env.require(
        async |env| {
            env.exists(
                upper_chains.into_iter().zip(children_alternatives),
                async |env, (upper_chain, mut child_alternative)| {
                    sub_chains(
                        env,
                        &mut child_alternative,
                        lower_chain,
                        upper_chain,
                        or_else,
                    )
                    .await
                },
            )
            .await
        },
        |env| or_else.report(env, Because::JustSo),
    )
    .await
}

#[boxed_async_fn]
async fn sub_chains<'db>(
    env: &mut Env<'db>,
    alternative: &mut Alternative<'_>,
    lower_chain: &[RedPermLink<'db>],
    upper_chain: &[RedPermLink<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<bool> {
    env.indent("sub_chains", &[&lower_chain, &upper_chain], async |env| {
        let db = env.db();
        match (lower_chain.split_first(), upper_chain.split_first()) {
            (None, _) => {
                // `my <= C`
                Ok(true)
            }

            (Some(_), None) => {
                let lower_term = RedPermLink::chain_to_perm(db, lower_chain);
                env.if_required(
                    alternative,
                    async |env| require_term_is_my(env, lower_term.into(), or_else).await,
                    async |env| term_is_provably_my(env, lower_term.into()).await,
                )
                .await
            }

            (Some((&lien0, c0)), Some((&lien1, c1))) => {
                sub_chains1(env, alternative, lien0, c0, lien1, c1, or_else).await
            }
        }
    })
    .await
}

#[boxed_async_fn]
async fn sub_chains1<'db>(
    env: &mut Env<'db>,
    alternative: &mut Alternative<'_>,
    lower_chain_head: RedPermLink<'db>,
    lower_chain_tail: &[RedPermLink<'db>],
    upper_chain_head: RedPermLink<'db>,
    upper_chain_tail: &[RedPermLink<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<bool> {
    let db = env.db();
    match (
        lower_chain_head,
        lower_chain_tail,
        upper_chain_head,
        upper_chain_tail,
    ) {
        (RedPermLink::Error(reported), _, _, _) | (_, _, RedPermLink::Error(reported), _) => {
            Err(reported)
        }

        (RedPermLink::Infer(v0), c0, _, _) => {
            if c0.is_empty() {
                env.if_required(
                    alternative,
                    async |env| {
                        require_upper_chain(env, v0, upper_chain_head, upper_chain_tail, or_else)
                            .await
                    },
                    async |env| {
                        splice_lower_bound(env, v0, c0, upper_chain_head, upper_chain_tail, or_else)
                            .await
                    },
                )
                .await
            } else {
                splice_lower_bound(env, v0, c0, upper_chain_head, upper_chain_tail, or_else).await
            }
        }

        (_, _, RedPermLink::Infer(v1), c1) => {
            if c1.is_empty() {
                env.if_required(
                    alternative,
                    async |env| {
                        require_lower_chain(env, upper_chain_head, upper_chain_tail, v1, or_else)
                            .await
                    },
                    async |env| {
                        splice_upper_bound(env, lower_chain_head, lower_chain_tail, v1, c1, or_else)
                            .await
                    },
                )
                .await
            } else {
                splice_upper_bound(env, lower_chain_head, lower_chain_tail, v1, c1, or_else).await
            }
        }

        (RedPermLink::Our, [], head1, tail1) => {
            // `our <= C1 if C1 is copy`
            let perm1 = RedPermLink::head_tail_to_perm(db, head1, tail1);
            env.if_required(
                alternative,
                async |env| require_term_is_copy(env, perm1.into(), or_else).await,
                async |env| term_is_provably_copy(env, perm1.into()).await,
            )
            .await
        }
        (RedPermLink::Our, c0, RedPermLink::Our, c1) => {
            // `(our C0) <= (our C1) if C0 <= C1`
            sub_chains(env, alternative, c0, c1, or_else).await
        }
        (RedPermLink::Our, _, RedPermLink::Leased(_), _) => Ok(false),
        (RedPermLink::Our, _, RedPermLink::Shared(_), _) => Ok(false),
        (RedPermLink::Our, _, RedPermLink::Var(_), _) => Ok(false),

        (RedPermLink::Leased(_), _, RedPermLink::Our, _) => Ok(false),
        (RedPermLink::Leased(place0), c0, RedPermLink::Leased(place1), c1) => {
            // * `(leased[place0] C0) <= (leased[place1] C1) if place1 <= place0 && C0 <= C1`
            if place0.is_covered_by(db, place1) {
                sub_chains(env, alternative, c0, c1, or_else).await
            } else {
                Ok(false)
            }
        }
        (RedPermLink::Leased(_), _, RedPermLink::Shared(_), _) => Ok(false),
        (RedPermLink::Leased(_), _, RedPermLink::Var(_), _) => Ok(false),

        (RedPermLink::Shared(place0), c0, RedPermLink::Our, [lien1, c1 @ ..]) => {
            // * `(shared[place0] C0) <= (our C1) if (leased[place0] C0) <= C1`
            sub_chains1(
                env,
                alternative,
                RedPermLink::Leased(place0),
                c0,
                *lien1,
                c1,
                or_else,
            )
            .await
        }
        (RedPermLink::Shared(_), _, RedPermLink::Our, []) => {
            // See above rule: if C1 is [] then `leased[place0] C0 <= []` will also be false.
            Ok(false)
        }
        (RedPermLink::Shared(place0), c0, RedPermLink::Shared(place1), c1) => {
            // * `(shared[place0] C0) <= (shared[place1] C1) if place1 <= place0 && C0 <= C1`
            if place0.is_covered_by(db, place1) {
                sub_chains(env, alternative, c0, c1, or_else).await
            } else {
                Ok(false)
            }
        }
        (RedPermLink::Shared(_), _, RedPermLink::Leased(_), _) => Ok(false),
        (RedPermLink::Shared(_), _, RedPermLink::Var(_), _) => Ok(false),

        (RedPermLink::Var(v0), [], RedPermLink::Our, []) => {
            // `X <= our`
            Ok(env.var_is_declared_to_be(v0, Predicate::Copy)
                && env.var_is_declared_to_be(v0, Predicate::Owned))
        }
        (RedPermLink::Var(_), _, RedPermLink::Our, _) => Ok(false),
        (RedPermLink::Var(v0), c0, RedPermLink::Var(v1), c1) => {
            // * `X C0 <= X C1 if C0 <= C1`
            if v0 == v1 {
                sub_chains(env, alternative, c0, c1, or_else).await
            } else {
                Ok(false)
            }
        }
        (RedPermLink::Var(_), _, RedPermLink::Leased(_), _) => Ok(false),
        (RedPermLink::Var(_), _, RedPermLink::Shared(_), _) => Ok(false),
    }
}

/// Covers the case where `L0 Ln <= ?U0`. This adds an upper bounding chain
/// to `?L0`.
async fn require_lower_chain<'db>(
    env: &mut Env<'db>,
    lower_head: RedPermLink<'db>,
    lower_tail: &[RedPermLink<'db>],
    upper_head: InferVarIndex,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let lower_chain = RedPerm::from_head_tail(env.db(), lower_head, lower_tail);

    let Some(or_else) =
        env.insert_chain_bound(upper_head, &lower_chain, Direction::FromBelow, or_else)
    else {
        return Ok(());
    };

    // If this is a new lower bound, spawn a task that will check that
    // there is at least one *upper bound* on the variable (either one
    // that currently exists or one that may be added in the future)
    // that is a superchain of this lower bound.
    env.runtime().spawn(
        env,
        TaskDescription::RequireLowerChain,
        async move |env| -> Result<(), dada_ir_ast::diagnostic::Reported> {
            env.log("RequireLowerChain", &[&lower_chain, &upper_head]);

            env.require_for_all_chain_bounds(
                upper_head,
                Direction::FromAbove,
                async |env, upper_chain| {
                    Alternative::the_future_never_comes(async |alternative| {
                        env.require(
                            async |env| {
                                sub_chains(env, alternative, &lower_chain, &upper_chain, &or_else)
                                    .await
                            },
                            |env| or_else.report(env, Because::JustSo),
                        )
                        .await
                    })
                    .await
                },
            )
            .await
        },
    );

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
    env: &mut Env<'db>,
    lower_head: RedPermLink<'db>,
    lower_tail: &[RedPermLink<'db>],
    upper_head: InferVarIndex,
    upper_tail: &[RedPermLink<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<bool> {
    let lower_chain = RedPerm::from_head_tail(env.db(), lower_head, lower_tail);
    env.exists_chain_bound(
        upper_head,
        Direction::FromAbove,
        async |env, mut upper_chain| {
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
    env: &mut Env<'db>,
    lower_head: InferVarIndex,
    upper_head: RedPermLink<'db>,
    upper_tail: &[RedPermLink<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let upper_chain = RedPerm::from_head_tail(env.db(), upper_head, upper_tail);

    let Some(_or_else) =
        env.insert_chain_bound(lower_head, &upper_chain, Direction::FromAbove, or_else)
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
    env: &mut Env<'db>,
    lower_head: InferVarIndex,
    lower_tail: &[RedPermLink<'db>],
    upper_head: RedPermLink<'db>,
    upper_tail: &[RedPermLink<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<bool> {
    let upper_chain = RedPerm::from_head_tail(env.db(), upper_head, upper_tail);
    env.exists_chain_bound(
        lower_head,
        Direction::FromBelow,
        async |env, mut lower_chain| {
            Alternative::the_future_never_comes(async |alternative| {
                lower_chain.extend(lower_tail);
                sub_chains(env, alternative, &lower_chain, &upper_chain, or_else).await
            })
            .await
        },
    )
    .await
}
