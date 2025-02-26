use dada_ir_ast::diagnostic::Errors;
use dada_util::{boxed_async_fn, vecset::VecSet};

use crate::{
    check::{
        chains::{Chain, Lien},
        combinator::{exists, require, require_for_all},
        env::Env,
        inference::InferenceVarData,
        predicates::{
            Predicate, is_provably_copy::term_is_provably_copy, require_copy::require_term_is_copy,
            require_term_is_my, term_is_provably_my,
        },
        report::{Because, OrElse},
    },
    ir::indices::InferVarIndex,
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
        let mut root = Alternative::root();
        let children_alternatives = root.spawn_children(upper_chains.len());
        require(
            exists(
                upper_chains.into_iter().zip(children_alternatives),
                async |(upper_chain, mut child_alternative)| {
                    sub_chains(
                        env,
                        &mut child_alternative,
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

async fn require_lower_bound<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    lower_liens: &[Lien<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let lower_chain = Chain::from_links(env.db(), lower_liens);

    if !env.runtime().insert_lower_bound(infer, lower_chain) {
        return Ok(());
    }

    let lower_bounds = env
        .runtime()
        .with_inference_var_data(infer, |data| data.lower_chains().clone());

    // IDEA: spawn a task checking that there exists an upper-bound that is compatible with this

    Ok(())
}
async fn require_upper_bound<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    upper_liens: &[Lien<'db>],
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let upper_chain = Chain::from_links(env.db(), upper_liens);

    if !env.runtime().insert_upper_bound(infer, upper_chain) {
        return Ok(());
    }

    // IDEA: no work is needed here, I think?
    // IDEA: but we do want to check at some point that
    // overall we can reconstruct a real type...?

    Ok(())
}

async fn exists_upper_bound<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    mut op: impl AsyncFnMut(Chain<'db>, &mut Alternative<'_>) -> Errors<bool>,
) -> Errors<bool> {
    // Spawn out two alternatives. The first one will represent the current bound
    // we are exploring. The second one will remain untouched forever and represents
    // any future bounds that may appear. The key point is that we will never
    // consider `op` to be *required*.
    let mut root = Alternative::root();
    let mut children = root.spawn_children(2);

    loop {
        let mut observed = VecSet::new();
        let mut stack = vec![];
        extract_bounding_chains(env, infer, &mut observed, &mut stack, |data| {
            data.upper_chains()
        })
        .await;

        while let Some(chain) = stack.pop() {
            match op(chain, &mut children[0]).await {
                Ok(true) => return Ok(true),
                Ok(false) => (),
                Err(reported) => return Err(reported),
            }
        }
    }
}

async fn for_all_lower_bounds<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    mut op: impl AsyncFnMut(Chain<'db>, &mut Alternative<'_>) -> Errors<bool>,
) -> Errors<bool> {
    // Spawn out two alternatives. The first one will represent the current bound
    // we are exploring. The second one will remain untouched forever and represents
    // any future bounds that may appear. The key point is that we will never
    // consider `op` to be *required*.
    let mut root = Alternative::root();
    let mut children = root.spawn_children(2);

    loop {
        let mut observed = VecSet::new();
        let mut stack = vec![];
        extract_bounding_chains(env, infer, &mut observed, &mut stack, |data| {
            data.lower_chains()
        })
        .await;

        while let Some(chain) = stack.pop() {
            match op(chain, &mut children[0]).await {
                Ok(true) => (),
                Ok(false) => return Ok(false),
                Err(reported) => return Err(reported),
            }
        }
    }
}

async fn extract_bounding_chains<'db>(
    env: &Env<'db>,
    infer: InferVarIndex,
    observed: &mut VecSet<Chain<'db>>,
    stack: &mut Vec<Chain<'db>>,
    op: impl for<'a> Fn(&'a InferenceVarData<'db>) -> &'a VecSet<Chain<'db>>,
) {
    env.runtime()
        .loop_on_inference_var(infer, |data| {
            let chains = op(data);
            assert!(stack.is_empty());
            for chain in chains {
                if observed.insert(chain.clone()) {
                    stack.push(chain.clone());
                }
            }
            if !stack.is_empty() { Some(()) } else { None }
        })
        .await;
}
