//! Check that one type is a subtype of another assuming inference has completed.

use super::{
    Env,
    bound::Direction,
    chains::{FirstLink, Lien, LienChain, TyChainKind},
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

fn is_subtychain<'db>(env: &Env<'db>, sub_chain: &TyChain<'db>, sup_chain: &TyChain<'db>) -> bool {
    match (&sub_chain.kind, &sup_chain.kind) {
        (TyChainKind::Error(_), _) | (_, TyChainKind::Error(_)) => true,

        (TyChainKind::Never, TyChainKind::Never) => true,
        (TyChainKind::Never, _) | (_, TyChainKind::Never) => false,

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

fn is_sublienchain<'db>(
    env: &Env<'db>,
    sub_chain: &LienChain<'db>,
    sup_chain: &LienChain<'db>,
) -> bool {
    match (sub_chain.first_link(), sup_chain.first_link()) {
        // my <: P is true for all P is they are either copy or owned. Leased permissions do not have compatible layout.
        (FirstLink::My, _) => is_copy_chain(env, sup_chain) || is_owned_chain(env, sup_chain),

        // Nothing is a subpermission of `my` except for `my`.
        (FirstLink::Lien(_), FirstLink::My) => false,

        (FirstLink::Lien(sub_lien), FirstLink::Lien(sup_lien)) => match (sub_lien, sup_lien) {
            (Lien::Error(_), _) | (_, Lien::Error(_)) => true,

            (Lien::Our, Lien::Our) => {}

            (Lien::Shared(sym_place), Lien::Our) => todo!(),
            (Lien::Shared(sym_place), Lien::Shared(sym_place)) => todo!(),
            (Lien::Shared(sym_place), Lien::Leased(sym_place)) => todo!(),
            (Lien::Shared(sym_place), Lien::Var(sym_variable)) => todo!(),
            (Lien::Leased(sym_place), Lien::Our) => todo!(),
            (Lien::Leased(sym_place), Lien::Shared(sym_place)) => todo!(),
            (Lien::Leased(sym_place), Lien::Leased(sym_place)) => todo!(),
            (Lien::Leased(sym_place), Lien::Var(sym_variable)) => todo!(),
            (Lien::Var(sym_variable), Lien::Our) => todo!(),
            (Lien::Var(sym_variable), Lien::Shared(sym_place)) => todo!(),
            (Lien::Var(sym_variable), Lien::Leased(sym_place)) => todo!(),
            (Lien::Var(sym_variable), Lien::Var(sym_variable)) => todo!(),
        },
    }
}

fn is_copy_chain<'db>(env: &Env<'db>, chain: &LienChain<'db>) -> bool {
    match chain.first_link() {
        FirstLink::My => false,
        FirstLink::Lien(lien) => match lien {
            Lien::Our | Lien::Shared(_) | Lien::Error(_) => true,
            Lien::Leased(_) => false,
            Lien::Var(sym_variable) => env.is_copy_var(sym_variable),
        },
    }
}

fn is_owned_chain<'db>(env: &Env<'db>, chain: &LienChain<'db>) -> bool {
    match chain.first_link() {
        FirstLink::My => false,
        FirstLink::Lien(lien) => match lien {
            Lien::Our | Lien::Error(_) => true,
            Lien::Shared(_) | Lien::Leased(_) => false,
            Lien::Var(sym_variable) => env.is_owned_var(sym_variable),
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
