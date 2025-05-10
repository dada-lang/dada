//! "Chains" are a canonicalized form of types/permissions.
//! They can only be produced after inference is complete as they require enumerating the bounds of inference variables.
//! They are used in borrow checking and for producing the final version of each inference variable.

use dada_ir_ast::diagnostic::{Err, Errors, Reported};
use dada_util::SalsaSerialize;
use salsa::Update;
use serde::Serialize;

use crate::ir::{
    indices::{FromInfer, InferVarIndex},
    types::{SymGenericTerm, SymPerm, SymPlace, SymTy, SymTyKind, SymTyName},
    variables::SymVariable,
};

use super::{env::Env, predicates::Predicate};

pub mod lattice;
pub mod sub;

/// A "lien chain" is a list of permissions by which some data may have been reached.
/// An empty lien chain corresponds to owned data (`my`, in surface Dada syntax).
/// A lien chain like `shared[p] leased[q]` would correspond to data shared from a variable `p`
/// which in turn had data leased from `q` (which in turn owned the data).
#[derive(SalsaSerialize)]
#[salsa::interned(debug)]
pub(crate) struct RedPerm<'db> {
    #[return_ref]
    pub chains: Vec<RedChain<'db>>,
}

impl<'db> RedPerm<'db> {
    pub fn is_provably(self, env: &Env<'db>, predicate: Predicate) -> Errors<bool> {
        let chains = self.chains(env.db());
        assert!(!chains.is_empty());
        for chain in chains {
            if !chain.is_provably(env, predicate)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn is_our(self, env: &Env<'db>) -> Errors<bool> {
        Ok(self.is_provably(env, Predicate::Copy)? && self.is_provably(env, Predicate::Owned)?)
    }

    pub fn to_sym_perm(self, db: &'db dyn crate::Db) -> SymPerm<'db> {
        self.chains(db)
            .iter()
            .map(|&chain| chain.to_sym_perm(db))
            .reduce(|perm1, perm2| SymPerm::or(db, perm1, perm2))
            .unwrap()
    }
}

impl<'db> Err<'db> for RedPerm<'db> {
    fn err(db: &'db dyn dada_ir_ast::Db, reported: Reported) -> Self {
        RedPerm::new(db, vec![RedChain::err(db, reported)])
    }
}

#[derive(SalsaSerialize)]
#[salsa::interned(debug)]
pub(crate) struct RedChain<'db> {
    #[return_ref]
    pub links: Vec<RedLink<'db>>,
}

impl<'db> RedChain<'db> {
    pub fn our(db: &'db dyn crate::Db) -> Self {
        RedChain::new(db, [RedLink::Our])
    }

    pub fn is_provably(self, env: &Env<'db>, predicate: Predicate) -> Errors<bool> {
        let db = env.db();
        match predicate {
            Predicate::Copy => RedLink::are_copy(env, self.links(db)),
            Predicate::Move => RedLink::are_move(env, self.links(db)),
            Predicate::Owned => RedLink::are_owned(env, self.links(db)),
            Predicate::Lent => RedLink::are_lent(env, self.links(db)),
        }
    }

    fn to_sym_perm(self, db: &'db dyn crate::Db) -> SymPerm<'db> {
        self.links(db)
            .iter()
            .map(|&link| link.to_sym_perm(db))
            .reduce(|perm1, perm2| SymPerm::apply(db, perm1, perm2))
            .unwrap_or_else(|| SymPerm::my(db))
    }
}

impl<'db> Err<'db> for RedChain<'db> {
    fn err(db: &'db dyn dada_ir_ast::Db, reported: Reported) -> Self {
        RedChain::new(db, vec![RedLink::err(db, reported)])
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub(crate) enum RedLink<'db> {
    Our,
    Ref(Live, SymPlace<'db>),
    Mut(Live, SymPlace<'db>),
    Var(SymVariable<'db>),
    Err(Reported),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Live(pub bool);

impl<'db> RedLink<'db> {
    pub fn are_copy(env: &Env<'db>, links: &[Self]) -> Errors<bool> {
        let Some(first) = links.first() else {
            return Ok(false);
        };
        first.is_copy(env)
    }

    pub fn are_move(env: &Env<'db>, links: &[Self]) -> Errors<bool> {
        for link in links {
            if !link.is_move(env)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn are_owned(env: &Env<'db>, links: &[Self]) -> Errors<bool> {
        for link in links {
            if !link.is_owned(env)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn are_lent(env: &Env<'db>, links: &[Self]) -> Errors<bool> {
        for link in links {
            if !link.is_lent(env)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn is_owned(&self, env: &Env<'db>) -> Errors<bool> {
        match self {
            RedLink::Our => Ok(true),
            RedLink::Var(v) => Ok(env.var_is_declared_to_be(*v, Predicate::Owned)),
            RedLink::Ref(..) | RedLink::Mut(..) => Ok(false),
            RedLink::Err(reported) => Err(*reported),
        }
    }

    pub fn is_lent(&self, env: &Env<'db>) -> Errors<bool> {
        match self {
            RedLink::Ref(..) | RedLink::Mut(..) => Ok(true),
            RedLink::Var(v) => Ok(env.var_is_declared_to_be(*v, Predicate::Lent)),
            RedLink::Our => Ok(false),
            RedLink::Err(reported) => Err(*reported),
        }
    }

    pub fn is_move(&self, env: &Env<'db>) -> Errors<bool> {
        match self {
            RedLink::Mut(..) => Ok(true),
            RedLink::Var(v) => Ok(env.var_is_declared_to_be(*v, Predicate::Move)),
            RedLink::Our | RedLink::Ref(..) => Ok(false),
            RedLink::Err(reported) => Err(*reported),
        }
    }

    pub fn is_copy(&self, env: &Env<'db>) -> Errors<bool> {
        match self {
            RedLink::Our | RedLink::Ref(..) => Ok(true),
            RedLink::Var(v) => Ok(env.var_is_declared_to_be(*v, Predicate::Copy)),
            RedLink::Mut(..) => Ok(false),
            RedLink::Err(reported) => Err(*reported),
        }
    }

    pub fn to_sym_perm(self, db: &'db dyn crate::Db) -> SymPerm<'db> {
        match self {
            RedLink::Our => SymPerm::our(db),
            RedLink::Ref(_, place) => SymPerm::shared(db, vec![place]),
            RedLink::Mut(_, place) => SymPerm::leased(db, vec![place]),
            RedLink::Var(v) => SymPerm::var(db, v),
            RedLink::Err(reported) => SymPerm::err(db, reported),
        }
    }
}

impl<'db> Err<'db> for RedLink<'db> {
    fn err(_db: &'db dyn dada_ir_ast::Db, reported: Reported) -> Self {
        RedLink::Err(reported)
    }
}

/// A "red(uced) type"-- captures just the
/// type layout part of a [`SymGenericTerm`][].
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Hash, Update, Serialize)]
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

impl<'db> Err<'db> for RedTy<'db> {
    fn err(_db: &'db dyn crate::Db, reported: Reported) -> Self {
        RedTy::Error(reported)
    }
}

impl<'db> RedTy<'db> {
    pub fn to_sym_ty(self, db: &'db dyn crate::Db) -> SymTy<'db> {
        match self {
            RedTy::Error(reported) => SymTy::err(db, reported),
            RedTy::Named(name, terms) => SymTy::named(db, name, terms),
            RedTy::Never => SymTy::never(db),
            RedTy::Infer(var_index) => SymTy::infer(db, var_index),
            RedTy::Var(variable) => SymTy::var(db, variable),
            RedTy::Perm => panic!("unexpected RedTy (perm)"),
        }
    }
}
