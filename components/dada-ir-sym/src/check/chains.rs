//! "Chains" are a canonicalized form of types/permissions.
//! They can only be produced after inference is complete as they require enumerating the bounds of inference variables.
//! They are used in borrow checking and for producing the final version of each inference variable.

use std::collections::VecDeque;

use dada_ir_ast::diagnostic::{Err, Errors, Reported};
use dada_util::{boxed_async_fn, vecset::VecSet};
use futures::StreamExt;
use salsa::Update;

use crate::ir::{
    indices::InferVarIndex,
    types::{SymGenericTerm, SymPerm, SymPermKind, SymPlace, SymTy, SymTyKind, SymTyName},
    variables::SymVariable,
};

use super::Env;

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
    pub fn ty_chains(&self) -> impl Iterator<Item = TyChain<'_, 'db>> {
        self.chains.iter().map(|chain| TyChain {
            chain,
            ty: &self.ty,
        })
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
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct TyChain<'l, 'db> {
    chain: &'l Chain<'db>,
    ty: &'l RedTy<'db>,
}

/// A "lien chain" is a list of permissions by which some data may have been reached.
/// An empty lien chain corresponds to owned data (`my`, in surface Dada syntax).
/// A lien chain like `shared[p] leased[q]` would correspond to data shared from a variable `p`
/// which in turn had data leased from `q` (which in turn owned the data).
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Update)]
pub struct Chain<'db> {
    links: Vec<Lien<'db>>,
}

impl<'db> Chain<'db> {
    /// Create a new [`Chain`].
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

impl<'db> Err<'db> for Lien<'db> {
    fn err(_db: &'db dyn crate::Db, reported: Reported) -> Self {
        Lien::Error(reported)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Update)]
pub enum RedTy<'db> {
    Error(Reported),
    Named(SymTyName<'db>, Vec<SymGenericTerm<'db>>),
    Never,
    Var(SymVariable<'db>),
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

pub trait ToRedTerm<'db> {
    fn to_red_term(&self, db: &'db dyn crate::Db) -> RedTerm<'db>;
}

trait ToRedTy<'db> {
    fn to_red_ty(&self, db: &'db dyn crate::Db) -> RedTy<'db>;
}

impl<'db, T: ToRedTerm<'db>> ToRedTerm<'db> for &T {
    fn to_red_term(&self, db: &'db dyn crate::Db) -> RedTerm<'db> {
        T::to_red_term(self, db)
    }
}

impl<'db> ToRedTerm<'db> for SymGenericTerm<'db> {
    fn to_red_term(&self, db: &'db dyn crate::Db) -> RedTerm<'db> {
        match *self {
            SymGenericTerm::Type(sym_ty) => sym_ty.to_red_term(db),
            SymGenericTerm::Perm(sym_perm) => sym_perm.to_red_term(db),
            SymGenericTerm::Place(_) => panic!("cannot create a red term from a place"),
            SymGenericTerm::Error(reported) => RedTerm::err(db, reported),
        }
    }
}

impl<'db> ToRedTerm<'db> for SymTy<'db> {
    fn to_red_term(&self, db: &'db dyn crate::Db) -> RedTerm<'db> {
        todo!()
    }
}

impl<'db> ToRedTy<'db> for SymTy<'db> {
    fn to_red_ty(&self, db: &'db dyn crate::Db) -> RedTy<'db> {
        match *self.kind(db) {
            SymTyKind::Perm(sym_perm, sym_ty) => todo!(),
            SymTyKind::Named(sym_ty_name, ref sym_generic_terms) => todo!(),
            SymTyKind::Infer(infer_var_index) => todo!(),
            SymTyKind::Var(sym_variable) => todo!(),
            SymTyKind::Never => todo!(),
            SymTyKind::Error(reported) => RedTy::err(db, reported),
        }
    }
}

impl<'db> ToRedTerm<'db> for SymPerm<'db> {
    fn to_red_term(&self, db: &'db dyn crate::Db) -> RedTerm<'db> {}
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
    fn to_chains(&self, db: &'db dyn crate::Db, env: &Env<'db>) -> Vec<Chain<'db>>;
}

impl<'db> ToChains<'db> for SymPerm<'db> {
    fn to_chains(&self, db: &'db dyn crate::Db, env: &Env<'db>) -> Vec<Chain<'db>> {
        match self.kind(db) {
            SymPermKind::My => vec![Chain::my(db)],
            SymPermKind::Our => vec![Chain::our(db)],
            SymPermKind::Shared(sym_places) => {}
            SymPermKind::Leased(sym_places) => todo!(),
            SymPermKind::Apply(sym_perm, sym_perm1) => todo!(),
            SymPermKind::Infer(infer_var_index) => todo!(),
            SymPermKind::Var(sym_variable) => todo!(),
            SymPermKind::Error(reported) => todo!(),
        }
    }
}
