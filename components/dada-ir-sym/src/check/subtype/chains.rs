use dada_ir_ast::diagnostic::Errors;
use dada_util::{boxed_async_fn, vecset::VecSet};

use crate::check::{
    chains::{Chain, Lien},
    combinator::{exists, require, require_for_all},
    env::Env,
    predicates::{
        Predicate, is_provably_copy::term_is_provably_copy, require_copy::require_term_is_copy,
        require_term_is_my, term_is_provably_my,
    },
    report::{Because, OrElse},
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
    let db = env.db();

    require_for_all(lower_chains, async |lower_chain| {
        let root = Alternative::root();
        let children_alternatives = root.spawn_children(upper_chains.len());
        require(
            exists(
                upper_chains.into_iter().zip(children_alternatives),
                async |(upper_chain, child_alternative)| {
                    sub_chains(
                        env,
                        &child_alternative,
                        lower_chain.links(),
                        upper_chain.links(),
                        or_else,
                    )
                    .await
                },
            ),
            || {
                or_else.report(
                    db,
                    Because::NotSubChain(lower_chain.clone(), upper_chains.clone()),
                )
            },
        )
        .await
    })
    .await
}

#[boxed_async_fn]
async fn sub_chains<'a, 'db>(
    env: &'a Env<'db>,
    alternative: &Alternative<'_>,
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
    alternative: &Alternative<'_>,
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

        (Lien::Infer(v0), [], Lien::Infer(v1), []) => {
            // XXX relate v0 and v1
            todo!("{v0:?} {v1:?}")
        }

        (Lien::Infer(v0), c0, _, _) => {
            if c0.is_empty() {
                // XXX add bound to v0
                todo!("{v0:?} {c0:?}")
            } else {
                todo!("{v0:?} {c0:?}")
            }
        }

        (_, _, Lien::Infer(v1), c1) => {
            if c1.is_empty() {
                // XXX add bound to v0
                todo!("{v1:?} {c1:?}")
            } else {
                todo!("{v1:?} {c1:?}")
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
