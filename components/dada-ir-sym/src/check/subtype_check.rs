//! Check that one type is a subtype of another assuming inference has completed.

use super::{Env, bound::Direction};
use crate::{
    check::chains::{ToChain, TyChain},
    ir::types::SymTy,
};

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
            .any(|sup_chain| is_subchain(env, sub_chain, sup_chain))
    })
}

fn is_subchain<'db>(env: &Env<'db>, sub_chain: &TyChain<'db>, sup_chain: &TyChain<'db>) -> bool {
    todo!()
}
