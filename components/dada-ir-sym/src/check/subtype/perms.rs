use dada_ir_ast::diagnostic::Errors;

use crate::{
    check::{
        env::Env,
        inference::Direction,
        live_places::LivePlaces,
        red::{RedLink, RedPerm, lattice::glb_perms, sub::chain_sub_chain},
        report::{Because, OrElse},
        stream::Consumer,
        to_red::ToRedPerm,
    },
    ir::{
        indices::InferVarIndex,
        types::{SymPerm, SymPermKind},
    },
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
    let db = env.db();

    // This is all a bit subtle. Here is the problem.
    // We have permissions coming in that we can convert to red-permissions.
    // Once we've done that, comparing two red-permissions for subtyping is relatively easy
    // (see `require_red_perm_sub_red_perm`).
    //
    // The challenge comes in when there are inference variables involved.
    // There can be a lot of ambiguity in this case. Consider something like
    // `P <: ?A ?B`. What do we attribute to `?A` vs `?B`?
    //
    // Right now we are doing the simplest thing that could possibly work.
    // There are definitely more sophisticated things we could do, it's not clear
    // yet whether it will matter.
    //
    // The strategy at present is to do three things at once:
    //
    // No matter what, we convert the lower/upper perms into red-perms and relate those.
    // If there are inference variables involved, this process will continue until
    // inference has completed, propagating any new bounds that appear on those variables.
    // We make no effort at present to avoid doing this multiple times in distinct tasks
    // for the same pairs of inference variables. That's for a future day.
    //
    // Concurrently, we check for cases where an inference variable appears on its
    // own in the lower/upper bound, e.g., `P <: ?A` or `?A <: Q`. These cases do not
    // have any ambiguities to worry about. We can just expand `P` (resp. `Q`)
    // to a set of red-permissions and add them to the lower (resp. upper) bounds of `?A`.
    // Note that in the case of `?A <: ?B` both of those cases apply simultaneously, and
    // so this serves to forward lower bounds from `?A` to `?B` and upper bounds from `?B` to `?A`.
    //
    // The main case that this does NOT handle is something like `our ?A <: our mut[x]`.
    // We could do in fact deduce that `?A` has an upper bound of `mut[x]` in this case,
    // but we are not smart enough. Instead, we'll just wait for any lower bounds on `?A` to show
    // up and compare them against `mut[x]`.

    env.require_all()
        .require(async |env| {
            let SymPermKind::Infer(lower_infer) = lower_perm.kind(db) else {
                return Ok(());
            };
            require_infer_bounded_by_perm(
                env,
                live_after,
                *lower_infer,
                Direction::FromAbove,
                upper_perm,
                or_else,
            )
            .await
        })
        .require(async |env| {
            let SymPermKind::Infer(upper_infer) = upper_perm.kind(db) else {
                return Ok(());
            };
            require_infer_bounded_by_perm(
                env,
                live_after,
                *upper_infer,
                Direction::FromBelow,
                lower_perm,
                or_else,
            )
            .await
        })
        .require(async |env| {
            require_perm_sub_perm(env, live_after, lower_perm, upper_perm, or_else).await
        })
        .finish()
        .await
}

async fn require_infer_bounded_by_perm<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    infer: InferVarIndex,
    direction: Direction,
    new_sym_bound: SymPerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    new_sym_bound
        .to_red_perm(
            env,
            live_after,
            direction,
            Consumer::new(async |env, new_red_bound: RedPerm<'db>| {
                match env.red_bound(infer, direction).peek_perm() {
                    Some((old_red_bound, old_or_else)) => {
                        match glb_perms(env, old_red_bound, new_red_bound) {
                            Some(red_perm_glb) => {
                                env.red_bound(infer, direction)
                                    .set_perm(red_perm_glb, or_else);
                            }
                            None => {
                                or_else.report(
                                    env,
                                    Because::InferredPermBound(
                                        direction,
                                        old_red_bound,
                                        old_or_else,
                                    ),
                                );
                            }
                        }
                    }

                    None => {
                        env.red_bound(infer, direction)
                            .set_perm(new_red_bound, or_else);
                    }
                }

                Ok(())
            }),
        )
        .await
}

async fn require_perm_sub_perm<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    lower_perm: SymPerm<'db>,
    upper_perm: SymPerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    lower_perm
        .to_red_perm(
            env,
            live_after,
            Direction::FromBelow,
            Consumer::new(async |env, lower_red_perm: RedPerm<'db>| {
                upper_perm
                    .to_red_perm(
                        env,
                        live_after,
                        Direction::FromAbove,
                        Consumer::new(async |env, upper_red_perm: RedPerm<'db>| {
                            require_red_perm_sub_red_perm(
                                env,
                                lower_red_perm,
                                upper_red_perm,
                                or_else,
                            )
                        }),
                    )
                    .await
            }),
        )
        .await
}

fn require_red_perm_sub_red_perm<'db>(
    env: &mut Env<'db>,
    lower_perm: RedPerm<'db>,
    upper_perm: RedPerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();

    // Require that forall lower there exists an upper where lower <= upper
    'lower: for &lower_chain in lower_perm.chains(db) {
        for &upper_chain in upper_perm.chains(db) {
            if chain_sub_chain(env, lower_chain, upper_chain)? {
                continue 'lower;
            }
        }

        // No suitable upper chain for `lower_chain`
        return Err(or_else.report(env, Because::JustSo));
    }

    Ok(())
}
