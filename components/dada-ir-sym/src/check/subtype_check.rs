//! Check that one type is a subtype of another assuming inference has completed.

use super::{Env, bound::Direction, chains::LienChain};
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
    todo!()
}

fn is_sublienchain<'db>(
    env: &Env<'db>,
    sub_chain: &LienChain<'db>,
    sup_chain: &LienChain<'db>,
) -> bool {
    todo!()
}

pub fn is_subplace<'db>(env: &Env<'db>, sub: SymPlace<'db>, sup: SymPlace<'db>) -> bool {
    sup.covers(env.db(), sub)
}
