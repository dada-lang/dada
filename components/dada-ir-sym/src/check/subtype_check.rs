//! Check that one type is a subtype of another assuming inference has completed.

use super::{
    Env,
    bound::Direction,
    chains::{Lien, LienChain, TyChainKind},
    resolve::Variance,
};
use crate::{
    check::chains::{ToChain, TyChain},
    ir::types::{SymGenericTerm, SymPerm, SymPlace, SymTy},
};

pub fn is_subterm<'db>(env: &Env<'db>, sub: SymGenericTerm<'db>, sup: SymGenericTerm<'db>) -> bool {
    match (sub, sup) {
        (SymGenericTerm::Error(_), _) | (_, SymGenericTerm::Error(_)) => true,

        (SymGenericTerm::Type(sub), SymGenericTerm::Type(sup)) => is_subtype(env, sub, sup),
        (SymGenericTerm::Type(_), _) | (_, SymGenericTerm::Type(_)) => {
            unreachable!("kind mismatch")
        }

        (SymGenericTerm::Perm(sub), SymGenericTerm::Perm(sup)) => is_subperm(env, sub, sup),
        (SymGenericTerm::Perm(_), _) | (_, SymGenericTerm::Perm(_)) => {
            unreachable!("kind mismatch")
        }

        (SymGenericTerm::Place(sub), SymGenericTerm::Place(sup)) => is_subplace(env, sub, sup),
    }
}

pub fn is_subtype<'db>(env: &Env<'db>, sub: SymTy<'db>, sup: SymTy<'db>) -> bool {
    let (sub_chains, sup_chains) = env.runtime().assert_check_complete(async {
        (
            ToChain::new(env)
                .into_ty_chains(sub, Direction::UpperBoundedBy)
                .await,
            ToChain::new(env)
                .into_ty_chains(sup, Direction::LowerBoundedBy)
                .await,
        )
    });

    sub_chains.iter().all(|sub_chain| {
        sup_chains
            .iter()
            .any(|sup_chain| is_subtychain(env, sub_chain, sup_chain))
    })
}

/// `a` is a *subpermission* of `b` if
///
/// 1. `a` *gives* a superset of `b`'s permissions to the place they are applied to
/// 2. `a` *imposes* a subset of `b`'s restrictions on other places
/// 3. `a` and `b` are 'layout compatible' -- i.e., for all types `T`, `a T` and `b T` have the same layout
pub fn is_subperm<'db>(env: &Env<'db>, sub: SymPerm<'db>, sup: SymPerm<'db>) -> bool {
    let (sub_chains, sup_chains) = env.runtime().assert_check_complete(async {
        (
            ToChain::new(env)
                .into_lien_chains(sub, Direction::UpperBoundedBy)
                .await,
            ToChain::new(env)
                .into_lien_chains(sup, Direction::LowerBoundedBy)
                .await,
        )
    });

    sub_chains.iter().all(|sub_chain| {
        sup_chains
            .iter()
            .any(|sup_chain| is_sublienchain(env, sub_chain, sup_chain))
    })
}

/// `sub_ty_chains(env, live_after, a, b)` indicates a value of type `a`
/// can be safely converted to a value of type `b` in the environment `env`
/// assuming that the places in `live_after` are live at the point of conversion.
fn is_subtychain<'db>(env: &Env<'db>, sub_chain: &TyChain<'db>, sup_chain: &TyChain<'db>) -> bool {
    match (&sub_chain.kind, &sup_chain.kind) {
        (TyChainKind::Error(_), _) | (_, TyChainKind::Error(_)) => true,

        // `never` is a subtype of itself
        (TyChainKind::Never, TyChainKind::Never) => true,

        // `never` is not a subtype of anything else, it is not representation compatible
        (TyChainKind::Never, _) | (_, TyChainKind::Never) => false,

        // type variables are compatible only with themselves
        (TyChainKind::Var(a), TyChainKind::Var(b)) => {
            a == b && is_sublienchain(env, &sub_chain.lien, &sup_chain.lien)
        }
        (TyChainKind::Var(_), _) | (_, TyChainKind::Var(_)) => false,

        (TyChainKind::Named(a, args_a), TyChainKind::Named(b, args_b)) if a == b => {
            let variances = env.variances(*a);
            assert_eq!(variances.len(), args_a.len());
            assert_eq!(variances.len(), args_b.len());
            is_sublienchain(env, &sub_chain.lien, &sup_chain.lien)
                && variances
                    .iter()
                    .zip(args_a)
                    .zip(args_b)
                    .all(|((&v, &a), &b)| {
                        is_subgeneric(env, v, &sub_chain.lien, a, &sup_chain.lien, b)
                    })
        }

        // FIXME: enum classes and subtyping
        (TyChainKind::Named(_a, _args_a), TyChainKind::Named(_b, _args_b)) => false,
    }
}

/// `sub_lien_chains(env, live_after, a, b)` indicates a value of some type `a T`
/// can be safely converted to a value of type `b T` in the environment `env`
/// and assuming that the places in `live_after` are live at the point of conversion.
fn is_sublienchain<'db>(
    env: &Env<'db>,
    sub_chain: &LienChain<'db>,
    sup_chain: &LienChain<'db>,
) -> bool {
    match (sub_chain.links(), sup_chain.links()) {
        ([Lien::Error(_), ..], _) | (_, [Lien::Error(_), ..]) => true,

        // my <= P is true for all P is they are either copy or owned. Leased permissions do not have compatible layout.
        ([], _) => is_copy_chain(env, sup_chain) || is_owned_chain(env, sup_chain),

        // nothing is a subtype of `my`
        (_, []) => false,

        // our <= P is true for all P is they are copy.
        ([Lien::Our], _) => is_copy_chain(env, sup_chain),

        // otherwise, all sub-liens must be covered by super-liens.
        ([sub_lien, sub_liens @ ..], [sup_lien, sup_liens @ ..]) => {
            sub_liens.len() <= sup_liens.len()
                && lien_covered_by(env, *sub_lien, *sup_lien)
                && sub_liens
                    .iter()
                    .zip(sup_liens)
                    .all(|(sub_lien, sup_lien)| lien_covered_by(env, *sub_lien, *sup_lien))
        }
    }
}

/// A lien `a` is *covered by* a lien `b` if
///
/// 1. `a` *gives* a superset of `b`'s permissions to the place they are applied to
/// 2. `a` *imposes* a subset of `b`'s restrictions on other places
/// 3. `a` and `b` are 'layout compatible' -- i.e., for all types `T`, `a T` and `b T` have the same layout
///
/// Permissions can be `move` or `copy` and correspond to the columns in the permission matrix.
///
/// Restrictions correspond to read or read/write restrictions on places. e.g., `shared[p]` imposes a read
/// striction on `p`, meaning that only reads from `p` are allowed so long as a `shared[p]` value is live.
fn lien_covered_by<'db>(env: &Env<'db>, sub_lien: Lien<'db>, sup_lien: Lien<'db>) -> bool {
    match (sub_lien, sup_lien) {
        (_, Lien::Error(_)) => true,
        (Lien::Error(_), _) => true,

        // TRUE CASES

        // `our` is covered by anything meeting `copy` bound:
        // 1. `our` gives `copy` permissions and so does anything meeting `copy` bound
        // 2. `our` imposes no restrictions so it must be a subset of the restrictions imposed by `sup_lien`.
        // 3. everything `copy` is by value
        (Lien::Our, _) => is_copy_lien(env, sup_lien),

        // e.g., `shared[a.b]` is covered by `shared[a]`:
        // 1. both give `copy` permissions
        // 2. `shared[a.b]` imposes a read restriction on `a.b`
        //    but `shared[a]` imposes a read striction on all of `a`.
        // 3. everything `copy` (including `shared`) is by value
        (Lien::Shared(sub_place, ..), Lien::Shared(sup_place, ..)) => {
            sub_place.is_covered_by(env.db(), sup_place)
        }

        // e.g., `leased[a.b]` is covered by `leased[a]`:
        // 1. both give `move` permissions
        // 2. `leased[a.b]` imposes a read/write restriction on `a.b`
        //    but `leased[a]` imposes a read/write restriction on all of `a`.
        // 3. everything leased is by value
        (Lien::Leased(sub_place, ..), Lien::Leased(sup_place, ..)) => {
            sub_place.is_covered_by(env.db(), sup_place)
        }

        // VARIABLES
        //
        // Since variables stand for some unknown set of permissions that will ultimately
        // be substituted, they must be equivalent to one of the cases above.

        // a variable `a` is only known to give the same permissions and impose the same restrictions as itself
        (Lien::Var(sub_var), Lien::Var(sup_var)) => {
            // same variable obviously meets all conditions
            sub_var == sup_var

                // `copy + owned` on `sub_var` means it is `our`, so same as `(Our, _)` above
                || env.is_copy_var(sub_var) && env.is_owned_var(sub_var) && env.is_copy_var(sup_var)

                // only `my` is `move + owned`, maps to the equality case
                || env.is_move_var(sub_var)
                    && env.is_owned_var(sub_var)
                    && env.is_move_var(sup_var)
                    && env.is_owned_var(sup_var)

            // note that e.g. `move` <= `move + owned` is false because
            // that would admit `leased(_) <= my` which meets conditions 1 and 2 but fails condition 3.
        }

        // only `our` is `copy + owned`, maps to `(Our, _)` above
        (Lien::Var(v), Lien::Our) | (Lien::Var(v), Lien::Shared(_)) => {
            env.is_copy_var(v) && env.is_owned_var(v)
        }

        // FALSE CASES

        // Fails condition 2: `shared[a]` imposes restriction to only read `a` but `our` imposes no restrictions
        (Lien::Shared(..), Lien::Our) => false,

        // Fails condition 2: `shared[a]` imposes read restriction on local variable `a` but perm variables cannot
        // (they can only name things from other frames)
        (Lien::Shared(..), Lien::Var(_)) => false,

        // Fails condition 1: `shared[a]` gives copy permission but `leased[a]` does not
        (Lien::Shared(..), Lien::Leased(_)) => false,

        // Fails condition 2: `leased[a]` imposes read/write restriction on `a` but `our` does not
        (Lien::Leased(..), Lien::Our) => false,

        // Fails condition 2: `leased[a]` imposes read/write restriction on `a` but perm variables do not
        // (they can only name things from other frames)
        (Lien::Leased(..), Lien::Var(_)) => false,

        // Fails condition 2: `leased[a]` imposes read/write restriction on `a` but `shared[a]` only
        // imposes read restriction
        (Lien::Leased(..), Lien::Shared(_)) => false,

        // Fails condition 2 or 3:
        // a. If sub is `move + lent`, then fails condition 2, as it may impose arbitrary restrictions not imposed by super.
        // b. If sub is `my = move + owned`, then fails condition 3, as `my` and `leased` are not layout compatible.
        (Lien::Var(_), Lien::Leased(_)) => false,
    }
}

fn is_copy_chain<'db>(env: &Env<'db>, chain: &LienChain<'db>) -> bool {
    match chain.links() {
        [] => false,
        [lien, ..] => is_copy_lien(env, *lien),
    }
}

fn is_copy_lien<'db>(env: &Env<'db>, lien: Lien<'db>) -> bool {
    match lien {
        Lien::Our | Lien::Shared(_) | Lien::Error(_) => true,
        Lien::Leased(_) => false,
        Lien::Var(v) => env.is_copy_var(v),
    }
}

fn is_owned_chain<'db>(env: &Env<'db>, chain: &LienChain<'db>) -> bool {
    match chain.links() {
        [] => false,
        [lien, ..] => match lien {
            Lien::Our | Lien::Error(_) => true,
            Lien::Shared(_) | Lien::Leased(_) => false,
            Lien::Var(v) => env.is_owned_var(*v),
        },
    }
}

fn is_subgeneric<'db>(
    env: &Env<'db>,
    variance: Variance,
    sub_cx: &LienChain<'db>,
    sub_term: SymGenericTerm<'db>,
    sup_cx: &LienChain<'db>,
    sup_term: SymGenericTerm<'db>,
) -> bool {
    todo!()
}

pub fn is_subplace<'db>(env: &Env<'db>, sub: SymPlace<'db>, sup: SymPlace<'db>) -> bool {
    sup.covers(env.db(), sub)
}
