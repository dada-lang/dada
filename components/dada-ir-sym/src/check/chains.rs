//! "Chains" are a canonicalized form of types/permissions.
//! They can only be produced after inference is complete as they require enumerating the bounds of inference variables.
//! They are used in borrow checking and for producing the final version of each inference variable.

use dada_ir_ast::diagnostic::{Err, Errors, Reported};
use dada_util::vecset::VecSet;
use either::Either;
use salsa::Update;

use crate::ir::{
    indices::InferVarIndex,
    types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind, SymTyName},
    variables::SymVariable,
};

use super::{
    Env,
    places::PlaceTy,
    predicates::{
        Predicate, is_ktb_copy::place_is_ktb_copy, test_infer_is_known_to_be,
        test_var_is_known_to_be,
    },
};

/// A "red(uced) term" combines the possible permissions (a [`VecSet`] of [`Chain`])
/// with the type of the term (a [`RedTy`]). It can be used to represent either permissions or types.
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Update)]
pub struct RedTerm<'db> {
    chains: VecSet<Chain<'db>>,
    ty: RedTy<'db>,
}

impl<'db> RedTerm<'db> {
    /// Create a new [`RedTerm`].
    pub fn new(_db: &'db dyn crate::Db, chains: VecSet<Chain<'db>>, ty: RedTy<'db>) -> Self {
        Self { ty, chains }
    }

    /// Yield an iterator of [`TyChain`]s, pairing each [`Chain`] with the [`RedTy`].
    pub fn ty_chains(&self) -> Vec<TyChain<'_, 'db>> {
        self.chains
            .iter()
            .map(|chain| TyChain {
                liens: &chain.liens,
                ty: &self.ty,
            })
            .collect()
    }

    /// Get the type of the term.
    pub fn ty(&self) -> &RedTy<'db> {
        &self.ty
    }

    /// Get the chains of the term.
    pub fn chains(&self) -> &VecSet<Chain<'db>> {
        &self.chains
    }
}

impl<'db> Err<'db> for RedTerm<'db> {
    fn err(db: &'db dyn crate::Db, reported: Reported) -> Self {
        RedTerm::new(db, Default::default(), RedTy::err(db, reported))
    }
}

/// Combination of a [`Chain`] and a [`RedTy`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub struct TyChain<'l, 'db> {
    pub liens: &'l [Lien<'db>],
    pub ty: &'l RedTy<'db>,
}

impl<'l, 'db> TyChain<'l, 'db> {
    /// Get the first lien in the chain.
    pub fn head(self) -> Either<Lien<'db>, &'l RedTy<'db>> {
        match self.liens.first() {
            Some(&lien) => Either::Left(lien),
            None => Either::Right(self.ty),
        }
    }

    /// Get the tail of the chain.
    pub fn tail(self) -> Self {
        Self {
            liens: &self.liens[1..],
            ty: self.ty,
        }
    }
}

/// A "lien chain" is a list of permissions by which some data may have been reached.
/// An empty lien chain corresponds to owned data (`my`, in surface Dada syntax).
/// A lien chain like `shared[p] leased[q]` would correspond to data shared from a variable `p`
/// which in turn had data leased from `q` (which in turn owned the data).
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Update)]
pub struct Chain<'db> {
    liens: Vec<Lien<'db>>,
}

impl<'db> Chain<'db> {
    /// Create a new [`Chain`].
    fn new(_db: &'db dyn crate::Db, links: Vec<Lien<'db>>) -> Self {
        Self { liens: links }
    }

    /// Access a slice of the links in the chain.
    pub fn links(&self) -> &[Lien<'db>] {
        &self.liens
    }

    /// Create a "fully owned" lien chain.
    pub fn my(db: &'db dyn crate::Db) -> Self {
        Self::new(db, vec![])
    }

    /// Create a "shared ownership" lien chain.
    pub fn our(db: &'db dyn crate::Db) -> Self {
        Self::new(db, vec![Lien::Our])
    }

    /// Create a variable lien chain.
    pub fn var(db: &'db dyn crate::Db, v: SymVariable<'db>) -> Self {
        Self::new(db, vec![Lien::Var(v)])
    }

    /// Create an inference lien chain.
    pub fn infer(db: &'db dyn crate::Db, v: InferVarIndex) -> Self {
        Self::new(db, vec![Lien::Infer(v)])
    }

    /// Create a lien chain representing "shared from `place`".
    fn shared(db: &'db dyn crate::Db, places: SymPlace<'db>) -> Self {
        Self::new(db, vec![Lien::Shared(places)])
    }

    /// Create a lien chain representing "leased from `place`".
    fn leased(db: &'db dyn crate::Db, places: SymPlace<'db>) -> Self {
        Self::new(db, vec![Lien::Leased(places)])
    }

    /// Concatenate two lien chains; if `other` is copy, just returns `other`.
    async fn concat(&self, db: &'db dyn crate::Db, env: &Env<'db>, other: &Self) -> Errors<Self> {
        if other.is_copy(env).await? {
            Ok(other.clone())
        } else {
            let mut links = self.liens.clone();
            links.extend(other.liens.iter());
            Ok(Self::new(db, links))
        }
    }

    /// Check if the chain is copy.
    async fn is_copy(&self, env: &Env<'db>) -> Errors<bool> {
        for lien in &self.liens {
            if lien.is_copy(env).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl<'db> Err<'db> for Chain<'db> {
    fn err(db: &'db dyn crate::Db, reported: Reported) -> Self {
        Chain::new(db, vec![Lien::Error(reported)])
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

    /// Data given from a generic variable (could be a type or permission variable).
    Var(SymVariable<'db>),

    /// Data given from a inference variable.
    Infer(InferVarIndex),

    /// An error occurred while processing this lien.
    Error(Reported),
}

impl<'db> Lien<'db> {
    /// Check if the lien is copy.
    async fn is_copy(&self, env: &Env<'db>) -> Errors<bool> {
        match *self {
            Lien::Our | Lien::Shared(_) => Ok(true),
            Lien::Leased(_) => Ok(false),
            Lien::Var(v) => Ok(test_var_is_known_to_be(env, v, Predicate::Copy)),
            Lien::Infer(v) => Ok(test_infer_is_known_to_be(env, v, Predicate::Copy).await),
            Lien::Error(reported) => Err(reported),
        }
    }
}

impl<'db> Err<'db> for Lien<'db> {
    fn err(_db: &'db dyn crate::Db, reported: Reported) -> Self {
        Lien::Error(reported)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Update)]
pub enum RedTy<'db> {
    /// An error occurred while processing this type.
    Error(Reported),

    /// A named type.
    Named(SymTyName<'db>, Vec<SymGenericTerm<'db>>),

    /// Never type.
    Never,

    /// An inference variable.
    Infer(InferVarIndex),

    /// A variable.
    Var(SymVariable<'db>),

    /// A permission -- this variant occurs when we convert a [`SymPerm`] to a [`RedTerm`].
    Perm,
}

impl<'db> RedTy<'db> {
    /// Return ty chain kind for unit (0-arity tuple).
    pub fn unit(_db: &'db dyn crate::Db) -> Self {
        Self::Named(SymTyName::Tuple { arity: 0 }, vec![])
    }
}

impl<'db> Err<'db> for RedTy<'db> {
    fn err(_db: &'db dyn crate::Db, reported: Reported) -> Self {
        RedTy::Error(reported)
    }
}

/// Convert something to a [`RedTerm`].
pub trait ToRedTerm<'db> {
    async fn to_red_term(&self, db: &'db dyn crate::Db, env: &Env<'db>) -> RedTerm<'db>;
}

/// Convert something to a [`RedTy`].
pub trait ToRedTy<'db> {
    fn to_red_ty(&self, db: &'db dyn crate::Db) -> RedTy<'db>;
}

impl<'db, T: ToRedTerm<'db>> ToRedTerm<'db> for &T {
    async fn to_red_term(&self, db: &'db dyn crate::Db, env: &Env<'db>) -> RedTerm<'db> {
        T::to_red_term(self, db, env).await
    }
}

impl<'db> ToRedTerm<'db> for SymGenericTerm<'db> {
    async fn to_red_term(&self, db: &'db dyn crate::Db, env: &Env<'db>) -> RedTerm<'db> {
        match *self {
            SymGenericTerm::Type(ty) => ty.to_red_term(db, env).await,
            SymGenericTerm::Perm(perm) => perm.to_red_term(db, env).await,
            SymGenericTerm::Place(_) => panic!("cannot create a red term from a place"),
            SymGenericTerm::Error(reported) => RedTerm::err(db, reported),
        }
    }
}

impl<'db> ToRedTerm<'db> for SymTy<'db> {
    async fn to_red_term(&self, db: &'db dyn crate::Db, env: &Env<'db>) -> RedTerm<'db> {
        match self.to_chains(db, env).await {
            Ok(chains) => RedTerm {
                chains,
                ty: self.to_red_ty(db),
            },
            Err(reported) => RedTerm::err(db, reported),
        }
    }
}

impl<'db> ToRedTy<'db> for SymTy<'db> {
    fn to_red_ty(&self, db: &'db dyn crate::Db) -> RedTy<'db> {
        match *self.kind(db) {
            SymTyKind::Perm(_, sym_ty) => sym_ty.to_red_ty(db),
            SymTyKind::Named(n, ref g) => RedTy::Named(n, g.clone()),
            SymTyKind::Infer(infer) => RedTy::Infer(infer),
            SymTyKind::Var(v) => RedTy::Var(v),
            SymTyKind::Never => RedTy::Never,
            SymTyKind::Error(reported) => RedTy::err(db, reported),
        }
    }
}

impl<'db> ToRedTerm<'db> for SymPerm<'db> {
    async fn to_red_term(&self, db: &'db dyn crate::Db, env: &Env<'db>) -> RedTerm<'db> {
        match self.to_chains(db, env).await {
            Ok(chains) => RedTerm {
                chains,
                ty: RedTy::Perm,
            },
            Err(reported) => RedTerm::err(db, reported),
        }
    }
}

impl<'db> ToRedTy<'db> for SymPerm<'db> {
    fn to_red_ty(&self, db: &'db dyn crate::Db) -> RedTy<'db> {
        match *self.kind(db) {
            SymPermKind::Error(reported) => RedTy::err(db, reported),
            _ => RedTy::Perm,
        }
    }
}

trait ToChains<'db> {
    async fn to_chains(&self, db: &'db dyn crate::Db, env: &Env<'db>)
    -> Errors<VecSet<Chain<'db>>>;
}

impl<'db> ToChains<'db> for SymPerm<'db> {
    async fn to_chains(
        &self,
        db: &'db dyn crate::Db,
        env: &Env<'db>,
    ) -> Errors<VecSet<Chain<'db>>> {
        let mut output = VecSet::new();
        match *self.kind(db) {
            SymPermKind::My => {
                output.insert(Chain::my(db));
            }
            SymPermKind::Our => {
                output.insert(Chain::our(db));
            }
            SymPermKind::Shared(ref places) => {
                for &place in places {
                    if place_is_ktb_copy(env, place).await.is_ok() {
                        output.extend(place.to_chains(db, env).await?);
                    } else {
                        output.insert(Chain::shared(db, place));
                    }
                }
            }
            SymPermKind::Leased(ref places) => {
                for &place in places {
                    if place_is_ktb_copy(env, place).await.is_ok() {
                        output.extend(place.to_chains(db, env).await?);
                    } else {
                        output.insert(Chain::leased(db, place));
                    }
                }
            }
            SymPermKind::Apply(lhs, rhs) => {
                let lhs_chains = lhs.to_chains(db, env).await?;
                let rhs_chains = rhs.to_chains(db, env).await?;
                for lhs_chain in &lhs_chains {
                    for rhs_chain in &rhs_chains {
                        output.insert(lhs_chain.concat(db, env, rhs_chain).await?);
                    }
                }
            }
            SymPermKind::Infer(v) => {
                output.insert(Chain::infer(db, v));
            }
            SymPermKind::Var(v) => {
                output.insert(Chain::var(db, v));
            }
            SymPermKind::Error(reported) => return Err(reported),
        }
        Ok(output)
    }
}

impl<'db> ToChains<'db> for SymPlace<'db> {
    async fn to_chains(
        &self,
        db: &'db dyn crate::Db,
        env: &Env<'db>,
    ) -> Errors<VecSet<Chain<'db>>> {
        let ty = self.place_ty(env).await;
        Ok(ty.to_chains(db, env).await?)
    }
}

impl<'db> ToChains<'db> for SymTy<'db> {
    async fn to_chains(
        &self,
        db: &'db dyn crate::Db,
        env: &Env<'db>,
    ) -> Errors<VecSet<Chain<'db>>> {
        let mut output = VecSet::new();
        match *self.kind(db) {
            SymTyKind::Perm(lhs, rhs) => {
                let lhs_chains = lhs.to_chains(db, env).await?;
                let rhs_chains = rhs.to_chains(db, env).await?;
                for lhs_chain in &lhs_chains {
                    for rhs_chain in &rhs_chains {
                        output.insert(lhs_chain.concat(db, env, rhs_chain).await?);
                    }
                }
            }
            SymTyKind::Never | SymTyKind::Named(..) | SymTyKind::Infer(_) | SymTyKind::Var(_) => {
                output.insert(Chain::my(db));
            }
            SymTyKind::Error(reported) => return Err(reported),
        }
        Ok(output)
    }
}
