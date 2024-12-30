//! "Chains" are a canonicalized form of types/permissions.
//! They can only be produced after inference is complete as they require enumerating the bounds of inference variables.
//! They are used in borrow checking and for producing the final version of each inference variable.

use std::{collections::VecDeque, future::Future, ops::AsyncFnMut};

use dada_ir_ast::diagnostic::{Err, Reported};
use salsa::Update;

use crate::ir::{
    types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind, SymTyName},
    variables::SymVariable,
};

use super::{bound::Direction, Env};

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Update)]
pub struct LienChain<'db> {
    links: Vec<Lien<'db>>,
}

impl<'db> LienChain<'db> {
    fn new(db: &'db dyn crate::Db, links: Vec<Lien<'db>>) -> Self {
        Self { links }
    }

    fn my(db: &'db dyn crate::Db) -> Self {
        Self::new(db, vec![])
    }

    fn our(db: &'db dyn crate::Db) -> Self {
        Self::new(db, vec![Lien::Our])
    }

    /// Add `lien` onto the chain; if `lien` is shared,
    /// this will drop whatever came before from the chain,
    /// but otherwise `lien` will be pushed on the end.
    fn apply(&mut self, env: &Env<'db>, lien: Lien<'db>) {
        if lien.is_shared_in_env(env) {
            self.links = vec![lien];
        } else {
            self.links.push(lien);
        }
    }
}

impl<'db> Err<'db> for LienChain<'db> {
    fn err(db: &'db dyn crate::Db, reported: Reported) -> Self {
        LienChain::new(db, vec![Lien::Error(reported)])
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Update)]
pub enum Lien<'db> {
    Our,
    Shared(SymPlace<'db>),
    Leased(SymPlace<'db>),
    Var(SymVariable<'db>),
    Error(Reported),
}

impl<'db> Lien<'db> {
    pub fn is_shared_in_env(&self, env: &Env<'db>) -> bool {
        match *self {
            Lien::Our => true,
            Lien::Shared(_) => true,
            Lien::Leased(_) => false,
            Lien::Var(sym_variable) => env.is_shared_var(sym_variable),
            Lien::Error(_) => false,
        }
    }

    /// True if `self` is equal to `other` (or more general, e.g., `shared(p)` 'covers' `shared(p.q)`).
    fn covers(self, db: &'db dyn crate::Db, other: Self) -> bool {
        match (self, other) {
            (Lien::Our, Lien::Our) => true,
            (Lien::Our, _) => false,

            (Lien::Shared(p1), Lien::Shared(p2)) => p1.covers(db, p2),
            (Lien::Shared(_), _) => false,

            (Lien::Leased(p1), Lien::Leased(p2)) => p1.covers(db, p2),
            (Lien::Leased(_), _) => false,

            (Lien::Var(v1), Lien::Var(v2)) => v1 == v2,
            (Lien::Var(v1), _) => false,

            (Lien::Error(_), _) | (_, Lien::Error(_)) => true,
        }
    }
}

impl<'db> Err<'db> for Lien<'db> {
    fn err(_db: &'db dyn crate::Db, reported: Reported) -> Self {
        Lien::Error(reported)
    }
}

#[salsa::tracked]
pub struct TyChain<'db> {
    pub lien: LienChain<'db>,
    pub kind: TyChainKind<'db>,
}

impl<'db> Err<'db> for TyChain<'db> {
    fn err(db: &'db dyn crate::Db, reported: Reported) -> Self {
        TyChain::new(
            db,
            LienChain::err(db, reported),
            TyChainKind::Error(reported),
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Update)]
pub enum TyChainKind<'db> {
    Error(Reported),
    Named(SymTyName<'db>, Vec<SymGenericTerm<'db>>),
    Never,
    Var(SymVariable<'db>),
}

/// A `Pair` is used during lien chain construction.
///
/// The challenge is that we have to fill in missing pieces from
/// the type the user gave.
///
/// For example, given `p: leased[q] leased[r] String`, all of
/// the following types would expand to `leased[p] leased[q] leased[r] String`:
///
/// 1. `leased[p] leased[q] leased[r] String`
/// 2. `leased[p] leased[r] String`
/// 3. `leased[p] String`
///
/// To manage this, when we see `leased[p]`, we mark the `leased[q] leased[r]` liens
/// that appear in `p`'s type as "pending". Then we return to the liens
/// from the input type (e.g., in case 2, that would be `leased[r]`). As we add the liens
/// from the input into the chain, we also add pending liens.
struct Pair<'db> {
    /// The chain of permissions that are "committed".
    chain: LienChain<'db>,

    /// *Pending* permissions are those that were implied by
    /// the type of some variable (e.g., `leased[`)
    pending: VecDeque<Lien<'db>>,
}

impl<'db> Pair<'db> {
    fn our(db: &'db dyn crate::Db) -> Self {
        Self {
            chain: LienChain::our(db),
            pending: Default::default(),
        }
    }

    /// Apply `lien` to the `chain`, pushing (some of the) pending liens first.
    fn apply(&mut self, env: &Env<'db>, lien: Lien<'db>) {
        while let Some(pending_lien) = self.pending.pop_front() {
            if lien.covers(env.db(), pending_lien) {
                break;
            }
            self.chain.apply(env, pending_lien);
        }

        self.chain.apply(env, lien);
    }

    fn into_lien_chain(mut self, env: &Env<'db>) -> LienChain<'db> {
        while let Some(pending_lien) = self.pending.pop_front() {
            self.chain.apply(env, pending_lien);
        }
        self.chain
    }
}

impl<'db> Err<'db> for Pair<'db> {
    fn err(db: &'db dyn crate::Db, reported: Reported) -> Self {
        Pair {
            chain: LienChain::err(db, reported),
            pending: Default::default(),
        }
    }
}

pub struct ToChain<'env, 'db> {
    db: &'db dyn crate::Db,
    env: &'env Env<'db>,
}

impl<'env, 'db> ToChain<'env, 'db> {
    pub fn new(env: &'env Env<'db>) -> Self {
        Self { db: env.db(), env }
    }

    // pub async fn ty_chains(
    //     &mut self,
    //     cx: LienChain<'db>,
    //     ty: SymTy<'db>,
    //     direction: Direction,
    // ) -> Vec<TyChain<'db>> {
    //     let pair = Pair {
    //         chain: cx,
    //         pending: Default::default(),
    //     };
    //     let mut chains = Vec::new();
    //     self.each_ty_chain(ty, pair, direction, async |chain| {
    //         chains.push(chain);
    //     })
    //     .await;
    //     chains
    // }

    // fn each_ty_chain(
    //     &mut self,
    //     ty: SymTy<'db>,
    //     pair: Pair<'db>,
    //     direction: Direction,
    //     mut yield_chain: impl AsyncFnMut(TyChain<'db>),
    // ) -> impl Future<Output = ()> {
    //     Box::pin(async move {
    //         match *ty.kind(self.db) {
    //             SymTyKind::Perm(sym_perm, sym_ty) => {}
    //             SymTyKind::Named(sym_ty_name, ref vec) => {}
    //             SymTyKind::Infer(infer_var_index) => todo!(),
    //             SymTyKind::Var(sym_variable) => {
    //                 yield_chain(TyChain::new(
    //                     self.db,
    //                     pair.into_lien_chain(self.env),
    //                     TyChainKind::Var(sym_variable),
    //                 ))
    //                 .await;
    //             }
    //             SymTyKind::Never => {
    //                 yield_chain(TyChain::new(
    //                     self.db,
    //                     LienChain::my(self.db),
    //                     TyChainKind::Never,
    //                 ))
    //                 .await;
    //             }
    //             SymTyKind::Error(reported) => yield_chain(TyChain::err(self.db, reported)).await,
    //         }
    //     })
    // }

    // fn perm_pairs(
    //     &mut self,
    //     pair: Pair<'db>,
    //     perm: SymPerm<'db>,
    //     yield_chain: impl AsyncFnMut(Pair<'db>),
    // ) -> impl Future<Output = ()> {
    //     Box::pin(async move {
    //         match *perm.kind(self.db) {
    //             SymPermKind::My => yield_chain(pair),
    //             SymPermKind::Our => yield_chain(Pair::our(self.db)),
    //             SymPermKind::Shared(ref vec) => {
    //                 self.shared_from_places(vec, yield_chain).await;
    //             }
    //             SymPermKind::Leased(ref vec) => {
    //                 self.leased_from_places(vec, yield_chain).await;
    //             }
    //             SymPermKind::Infer(infer_var_index) => todo!(),
    //             SymPermKind::Var(sym_variable) => todo!(),
    //             SymPermKind::Error(reported) => yield_chain(Pair::err(self.db, reported)),
    //         }
    //     })
    // }

    // async fn shared_from_places(
    //     &mut self,
    //     places: &[SymPlace<'db>],
    //     yield_chain: impl AsyncFnMut(Pair<'db>),
    // ) {
    //     for place in places {
    //         let perm = self.env.shared_perm(place);
    //         self.perm_pairs(pair, perm, yield_chain).await;
    //     }
    // }
}
