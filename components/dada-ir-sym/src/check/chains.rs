//! "Chains" are a canonicalized form of types/permissions.
//! They can only be produced after inference is complete as they require enumerating the bounds of inference variables.
//! They are used in borrow checking and for producing the final version of each inference variable.

use dada_ir_ast::diagnostic::{Err, Errors, Reported};
use dada_util::vecset::VecSet;
use salsa::Update;

use crate::ir::{
    indices::{FromInfer, InferVarIndex},
    types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind, SymTyName},
    variables::SymVariable,
};

use super::{
    Env,
    places::PlaceTy,
    predicates::{
        Predicate, is_provably_copy::place_is_provably_copy, test_infer_is_known_to_be,
        test_var_is_provably,
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

    /// Get the type of the term.
    pub fn ty(&self) -> &RedTy<'db> {
        &self.ty
    }

    /// Get the chains of the term.
    pub fn chains(&self) -> &VecSet<Chain<'db>> {
        &self.chains
    }

    pub fn into_chains(self) -> VecSet<Chain<'db>> {
        self.chains
    }
}

impl<'db> Err<'db> for RedTerm<'db> {
    fn err(db: &'db dyn crate::Db, reported: Reported) -> Self {
        RedTerm::new(db, Default::default(), RedTy::err(db, reported))
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

    pub fn from_head_tail(_db: &'db dyn crate::Db, head: Lien<'db>, tail: &[Lien<'db>]) -> Self {
        let mut liens = Vec::with_capacity(tail.len() + 1);
        liens.push(head);
        liens.extend(tail);
        Self { liens }
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
    async fn concat(&self, env: &Env<'db>, other: &Self) -> Errors<Self> {
        if other.is_copy(env).await? {
            Ok(other.clone())
        } else {
            let mut links = self.liens.clone();
            links.extend(other.liens.iter());
            Ok(Self::new(env.db(), links))
        }
    }

    /// Check if the chain is copy. Will block if this chain contains an inference variable.
    async fn is_copy(&self, env: &Env<'db>) -> Errors<bool> {
        for lien in &self.liens {
            if lien.is_copy(env).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn extend(&mut self, liens: &[Lien<'db>]) {
        self.liens.extend_from_slice(liens);
    }
}

impl<'db> std::ops::Deref for Chain<'db> {
    type Target = [Lien<'db>];

    fn deref(&self) -> &Self::Target {
        &self.liens
    }
}

impl<'db> Err<'db> for Chain<'db> {
    fn err(db: &'db dyn crate::Db, reported: Reported) -> Self {
        Chain::new(db, vec![Lien::Error(reported)])
    }
}

/// An individual unit in a [`LienChain`][], representing a particular way of reaching data.
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Update)]
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
            Lien::Var(v) => Ok(test_var_is_provably(env, v, Predicate::Copy)),
            Lien::Infer(v) => Ok(test_infer_is_known_to_be(env, v, Predicate::Copy).await),
            Lien::Error(reported) => Err(reported),
        }
    }

    /// Convert a (head, ..tail) to a permission.
    pub fn head_tail_to_perm(db: &'db dyn crate::Db, head: Self, tail: &[Self]) -> SymPerm<'db> {
        if tail.is_empty() {
            head.to_perm(db)
        } else {
            SymPerm::apply(db, head.to_perm(db), Self::chain_to_perm(db, tail))
        }
    }

    /// Convert a list of liens to a permission.
    pub fn chain_to_perm(db: &'db dyn crate::Db, liens: &[Self]) -> SymPerm<'db> {
        liens
            .iter()
            .map(|lien| lien.to_perm(db))
            .reduce(|lhs, rhs| SymPerm::apply(db, lhs, rhs))
            .unwrap_or_else(|| SymPerm::my(db))
    }

    /// Convert this lien to an equivalent [`SymPerm`].
    pub fn to_perm(self, db: &'db dyn crate::Db) -> SymPerm<'db> {
        match self {
            Lien::Our => SymPerm::our(db),
            Lien::Shared(place) => SymPerm::shared(db, vec![place]),
            Lien::Leased(place) => SymPerm::leased(db, vec![place]),
            Lien::Var(v) => SymPerm::var(db, v),
            Lien::Infer(v) => SymPerm::infer(db, v),
            Lien::Error(reported) => SymPerm::err(db, reported),
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
    pub fn display<'a>(&'a self, env: &'a Env<'db>) -> impl std::fmt::Display {
        struct Wrapper<'a, 'db> {
            ty: &'a RedTy<'db>,
            #[expect(dead_code)] // FIXME?
            env: &'a Env<'db>,
        }

        impl<'db> std::fmt::Display for Wrapper<'_, 'db> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match &self.ty {
                    RedTy::Error(_reported) => write!(f, "<error>"),
                    RedTy::Named(sym_ty_name, sym_generic_terms) => {
                        write!(f, "{}[{:?}]", sym_ty_name, sym_generic_terms)
                    }
                    RedTy::Never => write!(f, "!"),

                    // FIXME: do better by querying the env state
                    RedTy::Infer(v) => write!(f, "?{}", v.as_usize()),

                    RedTy::Var(sym_variable) => write!(f, "{}", sym_variable),
                    RedTy::Perm => write!(f, "<perm>"),
                }
            }
        }

        Wrapper { ty: self, env }
    }
}

impl<'db> Err<'db> for RedTy<'db> {
    fn err(_db: &'db dyn crate::Db, reported: Reported) -> Self {
        RedTy::Error(reported)
    }
}

/// Convert something to a [`RedTerm`].
pub trait ToRedTerm<'db> {
    async fn to_red_term(&self, env: &Env<'db>) -> RedTerm<'db>;
}

/// Convert something to a [`RedTy`] and an (optional) permission that is applied to that [`RedTy`][].
pub trait ToRedTy<'db> {
    fn to_red_ty(&self, env: &Env<'db>) -> (RedTy<'db>, Option<SymPerm<'db>>);
}

impl<'db, T: ToRedTerm<'db>> ToRedTerm<'db> for &T {
    async fn to_red_term(&self, env: &Env<'db>) -> RedTerm<'db> {
        T::to_red_term(self, env).await
    }
}

impl<'db> ToRedTerm<'db> for SymGenericTerm<'db> {
    async fn to_red_term(&self, env: &Env<'db>) -> RedTerm<'db> {
        match *self {
            SymGenericTerm::Type(ty) => ty.to_red_term(env).await,
            SymGenericTerm::Perm(perm) => perm.to_red_term(env).await,
            SymGenericTerm::Place(_) => panic!("cannot create a red term from a place"),
            SymGenericTerm::Error(reported) => RedTerm::err(env.db(), reported),
        }
    }
}

impl<'db> ToRedTerm<'db> for SymTy<'db> {
    async fn to_red_term(&self, env: &Env<'db>) -> RedTerm<'db> {
        match self.to_chains(env).await {
            Ok(chains) => RedTerm {
                chains,
                ty: self.to_red_ty(env).0,
            },
            Err(reported) => RedTerm::err(env.db(), reported),
        }
    }
}

impl<'db> ToRedTy<'db> for SymTy<'db> {
    fn to_red_ty(&self, env: &Env<'db>) -> (RedTy<'db>, Option<SymPerm<'db>>) {
        let db = env.db();
        match *self.kind(db) {
            SymTyKind::Perm(perm0, sym_ty) => match sym_ty.to_red_ty(env) {
                (red_ty, None) => (red_ty, Some(perm0)),
                (red_ty, Some(perm1)) => (red_ty, Some(SymPerm::apply(db, perm0, perm1))),
            },
            SymTyKind::Named(n, ref g) => (RedTy::Named(n, g.clone()), None),
            SymTyKind::Infer(infer) => {
                // every type inference variable has an associated permission inference variable,
                // so split that off
                let perm_infer = env.perm_infer(infer);
                (RedTy::Infer(infer), Some(SymPerm::infer(db, perm_infer)))
            }
            SymTyKind::Var(v) => (RedTy::Var(v), None),
            SymTyKind::Never => (RedTy::Never, None),
            SymTyKind::Error(reported) => (RedTy::err(db, reported), None),
        }
    }
}

impl<'db> ToRedTerm<'db> for SymPerm<'db> {
    async fn to_red_term(&self, env: &Env<'db>) -> RedTerm<'db> {
        match self.to_chains(env).await {
            Ok(chains) => RedTerm {
                chains,
                ty: RedTy::Perm,
            },
            Err(reported) => RedTerm::err(env.db(), reported),
        }
    }
}

impl<'db> ToRedTy<'db> for SymPerm<'db> {
    fn to_red_ty(&self, env: &Env<'db>) -> (RedTy<'db>, Option<SymPerm<'db>>) {
        let db = env.db();
        match *self.kind(db) {
            SymPermKind::Error(reported) => (RedTy::err(db, reported), None),
            _ => (RedTy::Perm, Some(*self)),
        }
    }
}

trait ToChains<'db> {
    async fn to_chains(&self, env: &Env<'db>) -> Errors<VecSet<Chain<'db>>>;
}

impl<'db> ToChains<'db> for SymPerm<'db> {
    async fn to_chains(&self, env: &Env<'db>) -> Errors<VecSet<Chain<'db>>> {
        Box::pin(async move {
            let mut output = VecSet::new();
            let db = env.db();
            match *self.kind(db) {
                SymPermKind::My => {
                    output.insert(Chain::my(db));
                }
                SymPermKind::Our => {
                    output.insert(Chain::our(db));
                }
                SymPermKind::Shared(ref places) => {
                    for &place in places {
                        if place_is_provably_copy(env, place).await.is_ok() {
                            output.extend(place.to_chains(env).await?);
                        } else {
                            output.insert(Chain::shared(env.db(), place));
                        }
                    }
                }
                SymPermKind::Leased(ref places) => {
                    for &place in places {
                        if place_is_provably_copy(env, place).await.is_ok() {
                            output.extend(place.to_chains(env).await?);
                        } else {
                            output.insert(Chain::leased(db, place));
                        }
                    }
                }
                SymPermKind::Apply(lhs, rhs) => {
                    let lhs_chains = lhs.to_chains(env).await?;
                    let rhs_chains = rhs.to_chains(env).await?;
                    for lhs_chain in &lhs_chains {
                        for rhs_chain in &rhs_chains {
                            output.insert(lhs_chain.concat(env, rhs_chain).await?);
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
        })
        .await
    }
}

impl<'db> ToChains<'db> for SymPlace<'db> {
    async fn to_chains(&self, env: &Env<'db>) -> Errors<VecSet<Chain<'db>>> {
        let ty = self.place_ty(env).await;
        Ok(ty.to_chains(env).await?)
    }
}

impl<'db> ToChains<'db> for SymTy<'db> {
    async fn to_chains(&self, env: &Env<'db>) -> Errors<VecSet<Chain<'db>>> {
        Box::pin(async move {
            let mut output = VecSet::new();
            let db = env.db();
            match *self.kind(db) {
                SymTyKind::Perm(lhs, rhs) => {
                    let lhs_chains = lhs.to_chains(env).await?;
                    let rhs_chains = rhs.to_chains(env).await?;
                    for lhs_chain in &lhs_chains {
                        for rhs_chain in &rhs_chains {
                            output.insert(lhs_chain.concat(env, rhs_chain).await?);
                        }
                    }
                }
                SymTyKind::Infer(infer) => {
                    output.insert(Chain::infer(db, env.perm_infer(infer)));
                }
                SymTyKind::Never | SymTyKind::Named(..) | SymTyKind::Var(_) => {
                    output.insert(Chain::my(db));
                }
                SymTyKind::Error(reported) => return Err(reported),
            }
            Ok(output)
        })
        .await
    }
}
