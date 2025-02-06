//! "Chains" are a canonicalized form of types/permissions.
//! They can only be produced after inference is complete as they require enumerating the bounds of inference variables.
//! They are used in borrow checking and for producing the final version of each inference variable.

use std::collections::VecDeque;

use dada_ir_ast::diagnostic::{Err, Reported};
use dada_util::boxed_async_fn;
use futures::StreamExt;
use salsa::Update;

use crate::ir::{
    types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind, SymTyName},
    variables::SymVariable,
};

use super::{Env, bounds::Direction};

/// A "lien chain" is a list of permissions by which some data may have been reached.
/// An empty lien chain corresponds to owned data (`my`, in surface Dada syntax).
/// A lien chain like `shared[p] leased[q]` would correspond to data shared from a variable `p`
/// which in turn had data leased from `q` (which in turn owned the data).
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Update)]
pub struct LienChain<'db> {
    links: Vec<Lien<'db>>,
}

impl<'db> LienChain<'db> {
    fn new(_db: &'db dyn crate::Db, links: Vec<Lien<'db>>) -> Self {
        Self { links }
    }

    /// Access a slice of the links in the chain.
    pub fn links(&self) -> &[Lien<'db>] {
        &self.links
    }

    /// Create a "fully owned" lien chain.
    pub fn my(db: &'db dyn crate::Db) -> Self {
        Self::new(db, vec![])
    }

    /// Create a "shared ownership" lien chain.
    pub fn our(db: &'db dyn crate::Db) -> Self {
        Self::new(db, vec![Lien::Our])
    }

    /// Create a lien chain representing "shared from `place`".
    ///
    fn shared(db: &'db dyn crate::Db, places: SymPlace<'db>) -> Self {
        Self::new(db, vec![Lien::Shared(places)])
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

/// An individual unit in a [`LienChain`][], representing a particular way of reaching data.
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Update)]
pub enum Lien<'db> {
    /// Data mutually owned by many variables. This lien is always first in a chain.
    Our,

    /// Data shared from the given place. This lien is always first in a chain.
    Shared(SymPlace<'db>),

    /// Data leased from the given place.
    Leased(SymPlace<'db>),

    /// Data given from a generic permission variable.
    Var(SymVariable<'db>),

    /// An error occurred while processing this lien.
    Error(Reported),
}

impl<'db> Lien<'db> {
    pub fn is_shared_in_env(&self, env: &Env<'db>) -> bool {
        match *self {
            Lien::Our => true,
            Lien::Shared(_) => true,
            Lien::Leased(_) => false,
            Lien::Var(sym_variable) => env.is_copy_var(sym_variable),
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
            (Lien::Var(_), _) => false,

            (Lien::Error(_), _) => true,
        }
    }
}

impl<'db> Err<'db> for Lien<'db> {
    fn err(_db: &'db dyn crate::Db, reported: Reported) -> Self {
        Lien::Error(reported)
    }
}

/// A "type chain" describes a data type and the permission chain (`lien`) by which that data can be reached.
pub struct TyChain<'db> {
    pub lien: LienChain<'db>,
    pub kind: TyChainKind<'db>,
}

impl<'db> TyChain<'db> {
    pub fn new(_db: &'db dyn crate::Db, lien: LienChain<'db>, kind: TyChainKind<'db>) -> Self {
        Self { lien, kind }
    }
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

impl<'db> TyChainKind<'db> {
    /// Return ty chain kind for unit (0-arity tuple).
    pub fn unit(_db: &'db dyn crate::Db) -> Self {
        Self::Named(SymTyName::Tuple { arity: 0 }, vec![])
    }
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
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
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

    fn shared(db: &'db dyn crate::Db, place: SymPlace<'db>) -> Self {
        Self {
            chain: LienChain::shared(db, place),
            pending: Default::default(),
        }
    }

    /// Clone the pair and apply `lien` to it.
    fn with_lien_applied(&self, env: &Env<'db>, lien: Lien<'db>) -> Self {
        let mut pair = self.clone();
        pair.apply_lien(env, lien);
        pair
    }

    /// Apply `lien` to the `chain`, pushing (some of the) pending liens first.
    fn apply_lien(&mut self, env: &Env<'db>, lien: Lien<'db>) {
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

    /// Return a set of "type chains" bounding `ty` from the given `direction`.
    pub async fn into_ty_chains(self, ty: SymTy<'db>, direction: Direction) -> Vec<TyChain<'db>> {
        let mut chains = vec![];
        self.push_ty_chains(ty, direction, &mut chains).await;
        chains
    }

    /// Push a set of "type chains" bounding `ty` from the given `direction` into the given vector.
    pub async fn push_ty_chains(
        &self,
        ty: SymTy<'db>,
        direction: Direction,
        chains: &mut Vec<TyChain<'db>>,
    ) {
        self.push_ty_chains_in_cx(LienChain::my(self.db), ty, direction, chains)
            .await
    }

    /// Return a set of "lien chains" bounding `perm` from the given `direction`.
    pub async fn into_lien_chains(
        self,
        perm: SymPerm<'db>,
        direction: Direction,
    ) -> Vec<LienChain<'db>> {
        let mut chains = vec![];
        self.push_lien_chains(perm, direction, &mut chains).await;
        chains
    }

    /// Push a set of "type chains" bounding `ty` from the given `direction` into the given vector.
    pub async fn push_lien_chains(
        &self,
        perm: SymPerm<'db>,
        direction: Direction,
        chains: &mut Vec<LienChain<'db>>,
    ) {
        self.push_lien_chains_in_cx(LienChain::my(self.db), perm, direction, chains)
            .await
    }

    /// Return a set of "type chains" bounding `ty` from the given `direction`
    /// when it appears in the context of the permission chain `cx`.
    pub async fn push_ty_chains_in_cx(
        &self,
        cx: LienChain<'db>,
        ty: SymTy<'db>,
        direction: Direction,
        chains: &mut Vec<TyChain<'db>>,
    ) {
        let pair = Pair {
            chain: cx,
            pending: Default::default(),
        };
        self.each_ty_chain(ty, pair, direction, &mut async |chain| {
            chains.push(chain);
        })
        .await;
    }

    /// Return a set of "type chains" bounding `ty` from the given `direction`
    /// when it appears in the context of the permission chain `cx`.
    pub async fn push_lien_chains_in_cx(
        &self,
        cx: LienChain<'db>,
        perm: SymPerm<'db>,
        direction: Direction,
        chains: &mut Vec<LienChain<'db>>,
    ) {
        let pair = Pair {
            chain: cx,
            pending: Default::default(),
        };
        self.perm_pairs(pair, direction, perm, &mut async |pair| {
            chains.push(pair.into_lien_chain(self.env));
        })
        .await;
    }

    /// Invoke `yield_chain` with each type chain bounding `ty` from `direction` in the permission context `pair`.
    #[boxed_async_fn]
    async fn each_ty_chain(
        &self,
        ty: SymTy<'db>,
        pair: Pair<'db>,
        direction: Direction,
        yield_chain: &mut impl AsyncFnMut(TyChain<'db>),
    ) {
        match *ty.kind(self.db) {
            SymTyKind::Perm(sym_perm, sym_ty) => {
                self.perm_pairs(pair, direction, sym_perm, &mut async move |pair| {
                    self.each_ty_chain(sym_ty, pair, direction, yield_chain)
                        .await;
                })
                .await;
            }
            SymTyKind::Named(sym_ty_name, ref vec) => {
                yield_chain(TyChain::new(
                    self.db,
                    pair.into_lien_chain(self.env),
                    TyChainKind::Named(sym_ty_name, vec.clone()),
                ))
                .await;
            }
            SymTyKind::Infer(var) => {
                let mut bounds = self.env.transitive_ty_var_bounds(var, direction);
                while let Some(bound) = bounds.next().await {
                    self.each_ty_chain(bound, pair.clone(), direction, yield_chain)
                        .await;
                }
            }
            SymTyKind::Var(sym_variable) => {
                yield_chain(TyChain::new(
                    self.db,
                    pair.into_lien_chain(self.env),
                    TyChainKind::Var(sym_variable),
                ))
                .await;
            }
            SymTyKind::Never => {
                yield_chain(TyChain::new(
                    self.db,
                    LienChain::my(self.db),
                    TyChainKind::Never,
                ))
                .await;
            }
            SymTyKind::Error(reported) => yield_chain(TyChain::err(self.db, reported)).await,
        }
    }

    /// Invoke `yield_chain` with each permission pair bounding `perm` from `direction` in the permission context `pair`.
    #[boxed_async_fn]
    async fn perm_pairs(
        &self,
        mut pair: Pair<'db>,
        direction: Direction,
        perm: SymPerm<'db>,
        yield_chain: &mut impl AsyncFnMut(Pair<'db>),
    ) {
        match *perm.kind(self.db) {
            SymPermKind::My => yield_chain(pair).await,
            SymPermKind::Our => yield_chain(Pair::our(self.db)).await,
            SymPermKind::Shared(ref places) => {
                self.shared_from_places(places, yield_chain).await;
            }
            SymPermKind::Leased(ref places) => {
                self.leased_from_places(pair, places, yield_chain).await;
            }
            SymPermKind::Infer(_) => {
                let mut bounds = self.env.transitive_perm_bounds(perm, direction);
                while let Some(bound) = bounds.next().await {
                    self.perm_pairs(pair.clone(), direction, bound, yield_chain)
                        .await;
                }
            }
            SymPermKind::Var(sym_variable) => {
                let var_lien = Lien::Var(sym_variable);
                pair.apply_lien(self.env, var_lien);
                yield_chain(pair).await;
            }
            SymPermKind::Error(reported) => yield_chain(Pair::err(self.db, reported)).await,
            SymPermKind::Apply(left, right) => {
                self.perm_pairs(pair, direction, left, &mut async |left_pair| {
                    self.perm_pairs(left_pair, direction, right, yield_chain)
                        .await
                })
                .await
            }
        }
    }

    /// Invoke `yield_chain` with permission pair shared from `places`.
    async fn shared_from_places(
        &self,
        places: &[SymPlace<'db>],
        yield_chain: &mut impl AsyncFnMut(Pair<'db>),
    ) {
        for &place in places {
            yield_chain(Pair::shared(self.db, place)).await;
        }
    }

    /// Invoke `yield_chain` with permission pair leased from `places`.
    /// `pair` represents the chain leading up to the lease.
    async fn leased_from_places(
        &self,
        pair: Pair<'db>,
        places: &[SymPlace<'db>],
        yield_chain: &mut impl AsyncFnMut(Pair<'db>),
    ) {
        for &place in places {
            let leased_lien = Lien::Leased(place);
            yield_chain(pair.with_lien_applied(self.env, leased_lien)).await;
        }
    }
}
