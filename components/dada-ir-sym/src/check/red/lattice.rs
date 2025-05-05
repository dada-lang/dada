use std::collections::BTreeSet;

use dada_ir_ast::diagnostic::{Err, Errors};
use dada_util::Set;
use itertools::Itertools;

use crate::{
    check::{env::Env, predicates::Predicate},
    ir::types::SymPlace,
};

use super::{Live, RedChain, RedLink, RedPerm, sub::chain_sub_chain};

/// The **least upper bound** (LUB) of two permissions `(perm1, perm2)`.
/// The LUB `perm3` is a permission such that `perm1 <: perm3` and `perm1 <: perm3`
/// but also there is no other mutual upper bound `perm4` where `perm4 <: perm3`.
/// In other words, if `perm1` and `perm2` represent lower bounds in inference
/// (i.e., for some inference variable `?X`, `perm1 <: ?X` and `perm2 <: ?X`),
/// then `perm3` combines  those two bounds on `?X` into a single bound `perm3 <: ?X`
/// that must also hold (because there is no possible value for `?X` that wouldn't
/// satisfy it).
///
/// Computing the LUB is fairly trivial for us because we can just union the chains.
/// This is not necessarily the most compact representation of the LUB but it is correct.
/// We do some minimal simplification.
///
/// # Examples
///
/// * `lub_perms([our], [ref[x]]) = [ref[x]]`.
///   * The union would be `[our, ref[x]]` but just `[ref[x]]` is equivalent and more compact.
///     * Equivalent because `(our | ref[x]) <: ref[x]` and `ref[x] <: (our | ref[x]))`
/// * `lub_perms([ref[x.y]], [ref[x]]) = [ref[x]]`.
///   * Another example where the union `[ref[x], ref[x.y]]` contains redundancies.
/// * `lub_perms([ref[x.y]], [ref[x.z]]) = [ref[x.y], ref[x.z]]`.
///   * Union suffices.
/// * `lub_perms([ref[x.y]], [ref[x.z]]) = [ref[x.y], ref[x.z]]`.
pub fn lub_perms<'db>(env: &Env<'db>, perm1: RedPerm<'db>, perm2: RedPerm<'db>) -> RedPerm<'db> {
    let db = env.db();

    let mut dedup = Set::default();
    let mut candidates = perm1
        .chains(db)
        .iter()
        .chain(perm2.chains(db))
        .copied()
        .filter(|&c| dedup.insert(c))
        .collect::<Vec<_>>();

    match simplify(env, &mut candidates) {
        Ok(()) => (),
        Err(reported) => {
            return RedPerm::err(db, reported);
        }
    }

    RedPerm::new(db, candidates)
}

/// The **greatest lower bound** (GLB) of two permissions `(perm1, perm2)`, if it exists.
/// The GLB `perm3` is a permission such that `perm3 <: perm1` and `perm3 <: perm2`
/// but also there is no other mutual lower bound `perm4` where `perm3 <: perm4`.
/// In other words, if `perm1` and `perm2` represent upper bounds in inference
/// (i.e., for some inference variable `?X`, `?X <: perm1` and `?X <: perm2`),
/// then `perm3` combines  those two bounds on `?X` into a single bound `?X <: perm3`
/// that must also hold (because there is no possible value for `?X` that wouldn't
/// satisfy it).
///
/// # Examples
///
/// * `glb_perms([our], [ref[x]]) = [our]`
/// * `glb_perms([ref[x] mut[y.z]], [ref[x.y] mut[y]]) = [ref[x.y] mut[y.z]]`
pub fn glb_perms<'db>(
    env: &Env<'db>,
    perm1: RedPerm<'db>,
    perm2: RedPerm<'db>,
) -> Option<RedPerm<'db>> {
    GreatestLowerBound::glb(env, perm1, perm2).ok()
}

struct NoGlb;
type Glb<T> = Result<T, NoGlb>;

trait GreatestLowerBound<'db>: Sized {
    fn glb(env: &Env<'db>, l1: Self, l2: Self) -> Glb<Self>;
}

impl<'db> GreatestLowerBound<'db> for RedPerm<'db> {
    fn glb(env: &Env<'db>, l1: RedPerm<'db>, l2: RedPerm<'db>) -> Glb<RedPerm<'db>> {
        // The algorithm:
        //
        // * Compute a list of candidates L consisting of
        //   * all `glb(c1, c2)` that exist from
        //     * each pair of chains `(c1, c2)` in `(l1.chains x l2.chains)`
        // * Simplify the candidates by removing any candidate c1 where
        //   * there exists another candidate c2 and
        //     * c1 < c2
        //
        // If L is empty, there is no GLB. Otherwise, result is the simplified list L.
        //
        // Examples:
        //
        // * [[ref[a], ref[b.x]] vs [ref[b]]
        //   * Initial candidate list = [our, ref[b.x]] because
        //     * glb(ref[a], ref[b]) = our
        //     * glb(ref[b.x], ref[b]) = ref[b.x]
        //   * Simplified candidate list = [ref[b.x]]
        // * [[mut[a], mut[b.x]] vs [mut[b]]
        //   * Initial candidate list = [mut[b.x]] because
        //     * glb(mut[a], mut[b]) does not exist
        //     * glb(mut[b.x], mut[b]) = mut[b.x]
        //   * Simplified candidate list = [mut[b.x]]
        // * [[ref[b.x] mut[c]] vs [ref[b.x] mut[c.d], ref[b]]
        //   * Initial candidate list = [ref[b.x] mut[c.d]] because
        //     * glb([ref[b.x] mut[c], [ref[b.x] mut[c.d]) = [ref[b.x], mut[c.d]]
        //     * glb([ref[b.x] mut[c], [ref[b]) = error

        let db = env.db();

        let mut unique = Set::default();
        let mut candidates: Vec<RedChain<'db>> = l1
            .chains(db)
            .iter()
            .cartesian_product(l2.chains(db))
            .filter_map(|(&l1, &l2)| RedChain::glb(env, l1, l2).ok())
            .filter(|&c| unique.insert(c))
            .collect();

        // Remove any chain `c1` which is a subchain of some other chain `c2`.
        //
        // so e.g. if you have `[our, ref[x]]`, `our <: ref[x]`, so remove `our`.
        match simplify(env, &mut candidates) {
            Ok(()) => (),
            Err(reported) => {
                return Ok(RedPerm::err(db, reported));
            }
        }

        if candidates.is_empty() {
            Err(NoGlb)
        } else {
            Ok(RedPerm::new(db, candidates))
        }
    }
}

impl<'db> GreatestLowerBound<'db> for RedChain<'db> {
    fn glb(env: &Env<'db>, chain1: RedChain<'db>, chain2: RedChain<'db>) -> Glb<RedChain<'db>> {
        let db = env.db();
        let mut links1 = &chain1.links(db)[..];
        let mut links2 = &chain2.links(db)[..];

        let mut links_glb = vec![];
        loop {
            match (links1, links2) {
                ([], []) => break,

                ([RedLink::Our], o) | (o, [RedLink::Our]) => {
                    // The only way to get `RedLink::Our` as the only item in the list
                    // is if it is the first thing in the list.
                    assert!(links_glb.is_empty());
                    return match RedLink::are_copy(env, o) {
                        Ok(true) => Ok(RedChain::our(db)),
                        Ok(false) => Err(NoGlb),
                        Err(reported) => Ok(RedChain::err(db, reported)),
                    };
                }

                ([RedLink::Our, RedLink::Mut(..), tail1 @ ..], [RedLink::Our, tail2 @ ..])
                | ([RedLink::Our, tail2 @ ..], [RedLink::Our, RedLink::Mut(..), tail1 @ ..]) => {
                    links_glb.push(RedLink::Our);
                    links1 = tail1;
                    links2 = tail2;
                }

                (
                    [RedLink::Our, RedLink::Mut(live1, place1), tail1 @ ..],
                    [RedLink::Ref(live2, place2), tail2 @ ..],
                )
                | (
                    [RedLink::Ref(live2, place2), tail2 @ ..],
                    [RedLink::Our, RedLink::Mut(live1, place1), tail1 @ ..],
                ) => {
                    links_glb.push(match SymPlace::glb(env, *place1, *place2) {
                        Ok(place3) => RedLink::Ref(glb_live(*live1, *live2), place3),
                        Err(NoGlb) => RedLink::Our,
                    });

                    links1 = tail1;
                    links2 = tail2;
                }

                ([head1, tail1 @ ..], [head2, tail2 @ ..]) => {
                    links_glb.push(RedLink::glb(env, *head1, *head2)?);
                    links1 = tail1;
                    links2 = tail2;
                }

                ([], [_, ..]) | ([_, ..], []) => {
                    // Uneven lengths.
                    return Err(NoGlb);
                }
            }
        }

        Ok(RedChain::new(db, links_glb))
    }
}

impl<'db> GreatestLowerBound<'db> for RedLink<'db> {
    fn glb(env: &Env<'db>, l1: RedLink<'db>, l2: RedLink<'db>) -> Glb<RedLink<'db>> {
        match (l1, l2) {
            (RedLink::Err(reported), _) | (_, RedLink::Err(reported)) => Ok(RedLink::Err(reported)),

            (RedLink::Our, RedLink::Ref(..))
            | (RedLink::Ref(..), RedLink::Our)
            | (RedLink::Our, RedLink::Our) => Ok(RedLink::Our),

            (RedLink::Our, RedLink::Var(v)) | (RedLink::Var(v), RedLink::Our) => {
                if env.var_is_declared_to_be(v, Predicate::Copy) {
                    Ok(RedLink::Our)
                } else {
                    Err(NoGlb)
                }
            }

            (RedLink::Ref(live1, p1), RedLink::Ref(live2, p2)) => {
                match SymPlace::glb(env, p1, p2) {
                    Ok(p3) => Ok(RedLink::Ref(glb_live(live1, live2), p3)),
                    Err(NoGlb) => Ok(RedLink::Our),
                }
            }

            (RedLink::Mut(live1, p1), RedLink::Mut(live2, p2)) => {
                let p3 = SymPlace::glb(env, p1, p2)?;
                Ok(RedLink::Mut(glb_live(live1, live2), p3))
            }

            (RedLink::Var(v1), RedLink::Var(v2)) => {
                if v1 == v2 {
                    Ok(RedLink::Var(v1))
                } else {
                    Err(NoGlb)
                }
            }

            (RedLink::Ref(..), RedLink::Var(v)) | (RedLink::Var(v), RedLink::Ref(..)) => {
                // Subtle: we canonicalize vars to `Our` if we can
                debug_assert!(
                    !env.var_is_declared_to_be(v, Predicate::Copy)
                        || !env.var_is_declared_to_be(v, Predicate::Owned)
                );

                Err(NoGlb)
            }

            // No type is a subtype of both our/mut at same time.
            (RedLink::Our, RedLink::Mut(..)) | (RedLink::Mut(..), RedLink::Our) => Err(NoGlb),

            // No type is a subtype of both ref/mut at same time.
            (RedLink::Ref(..), RedLink::Mut(..)) | (RedLink::Mut(..), RedLink::Ref(..)) => {
                Err(NoGlb)
            }

            // No type is a subtype of both var/mut at same time.
            (RedLink::Mut(..), RedLink::Var(_)) | (RedLink::Var(_), RedLink::Mut(..)) => Err(NoGlb),
        }
    }
}

impl<'db> GreatestLowerBound<'db> for SymPlace<'db> {
    fn glb(env: &Env<'db>, p1: SymPlace<'db>, p2: SymPlace<'db>) -> Glb<SymPlace<'db>> {
        if p1.is_prefix_of(env.db(), p2) {
            Ok(p2)
        } else if p2.is_prefix_of(env.db(), p1) {
            Ok(p1)
        } else {
            Err(NoGlb)
        }
    }
}

fn glb_live(Live(l1): Live, Live(l2): Live) -> Live {
    Live(l1 || l2)
}

/// Remove each candidate `c1 \in candidates` where
/// there exists another candidates `c2 \in candidates`
/// and `c1 <: c2`
fn simplify<'db>(env: &Env<'db>, candidates: &mut Vec<RedChain<'db>>) -> Errors<()> {
    let mut redundant = Set::default();
    for [c1, c2] in (0..candidates.len()).array_combinations() {
        let c1 = candidates[c1];
        let c2 = candidates[c2];

        if chain_sub_chain(env, c1, c2)? {
            redundant.insert(c1);
        } else if chain_sub_chain(env, c2, c1)? {
            redundant.insert(c2);
        }
    }

    candidates.retain(|c| !redundant.contains(c));
    Ok(())
}
