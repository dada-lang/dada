use dada_ir_ast::diagnostic::Errors;

use crate::{
    check::{
        env::Env,
        inference::Direction,
        live_places::LivePlaces,
        red::{RedLink, RedPerm},
        report::{Because, OrElse},
        stream::Consumer,
        to_red::ToRedPerm,
    },
    ir::{
        indices::InferVarIndex,
        types::{SymPerm, SymPermKind, SymPlace},
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
            require_infer_sub_perm(env, live_after, *lower_infer, upper_perm, or_else).await
        })
        .require(async |env| {
            let SymPermKind::Infer(upper_infer) = upper_perm.kind(db) else {
                return Ok(());
            };
            require_perm_sub_infer(env, live_after, lower_perm, *upper_infer, or_else).await
        })
        .require(async |env| {
            require_perm_sub_perm(env, live_after, lower_perm, upper_perm, or_else).await
        })
        .finish()
        .await
}

async fn require_infer_sub_perm<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    lower_infer: InferVarIndex,
    upper_perm: SymPerm<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    upper_perm
        .to_red_perm(
            env,
            live_after,
            Direction::FromAbove,
            Consumer::new(async |env, upper_red_perm: RedPerm<'db>| {
                env.insert_red_perm_bound(
                    lower_infer,
                    upper_red_perm,
                    Direction::FromAbove,
                    or_else,
                );
                Ok(())
            }),
        )
        .await
}

async fn require_perm_sub_infer<'db>(
    env: &mut Env<'db>,
    live_after: LivePlaces,
    lower_perm: SymPerm<'db>,
    upper_infer: InferVarIndex,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    lower_perm
        .to_red_perm(
            env,
            live_after,
            Direction::FromBelow,
            Consumer::new(async |env, upper_red_perm: RedPerm<'db>| {
                env.insert_red_perm_bound(
                    upper_infer,
                    upper_red_perm,
                    Direction::FromBelow,
                    or_else,
                );
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
    if let Some(_unmatched_chain) = lower_perm.chains(db).iter().find(|lower_chain| {
        upper_perm.chains(db).iter().all(|upper_chain| {
            !test_red_links_sub_red_links(env, lower_chain.links(db), upper_chain.links(db))
        })
    }) {
        Err(or_else.report(env, Because::JustSo))
    } else {
        Ok(())
    }
}

fn test_red_links_sub_red_links<'db>(
    env: &Env<'db>,
    lower_links: &[RedLink<'db>],
    upper_links: &[RedLink<'db>],
) -> bool {
    macro_rules! rules {
        ($($pat:pat => $cond:expr,)*) => {
            match (lower_links, upper_links) {
                $(
                    $pat if $cond => true,
                )*
                _ => false,
            }
        };
    }

    rules! {
        ([], []) => true,

        ([RedLink::Our], links_u) => RedLink::are_copy(env, links_u),

        ([RedLink::Our, tail_l @ ..], [head_u, tail_u @ ..]) => {
            head_u.is_copy(env)
            && test_red_links_sub_red_links(env, tail_l, tail_u)
        },

        (
            [
                RedLink::RefLive(place_l) | RedLink::RefDead(place_l),
                tail_l @ ..,
            ],
            [
                RedLink::RefLive(place_u) | RedLink::RefDead(place_u),
                tail_u @ ..,
            ],
        )
        | (
            [
                RedLink::MutLive(place_l) | RedLink::MutDead(place_l),
                tail_l @ ..,
            ],
            [
                RedLink::MutLive(place_u) | RedLink::MutDead(place_u),
                tail_u @ ..,
            ],
        )
        | (
            [
                RedLink::RefLive(place_l) | RedLink::RefDead(place_l),
                tail_l @ ..,
            ],
            [
                RedLink::Our,
                RedLink::MutLive(place_u) | RedLink::MutDead(place_u),
                tail_u @ ..,
            ],
        ) => {
            test_place_sub_place(env, *place_l, *place_u)
            && test_red_links_sub_red_links(env, tail_l, tail_u)
        },

        ([RedLink::Var(var_l), tail_l @ ..], [RedLink::Var(var_u), tail_u @ ..]) => {
            var_l == var_u && test_red_links_sub_red_links(env, tail_l, tail_u)
        },


    }
}

fn test_place_sub_place<'db>(
    env: &Env<'db>,
    place_l: SymPlace<'db>,
    place_r: SymPlace<'db>,
) -> bool {
    place_l.is_covered_by(env.db(), place_r)
}
