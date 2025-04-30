//! "Chains" are a canonicalized form of types/permissions.
//! They can only be produced after inference is complete as they require enumerating the bounds of inference variables.
//! They are used in borrow checking and for producing the final version of each inference variable.

use dada_ir_ast::diagnostic::{Err, Errors, Reported};
use dada_util::SalsaSerialize;
use salsa::Update;
use serde::Serialize;

use crate::ir::{
    indices::InferVarIndex,
    types::{SymGenericTerm, SymPlace, SymTyName},
    variables::SymVariable,
};

use super::{env::Env, predicates::Predicate};

/// A "lien chain" is a list of permissions by which some data may have been reached.
/// An empty lien chain corresponds to owned data (`my`, in surface Dada syntax).
/// A lien chain like `shared[p] leased[q]` would correspond to data shared from a variable `p`
/// which in turn had data leased from `q` (which in turn owned the data).
#[derive(SalsaSerialize)]
#[salsa::interned(debug)]
pub struct RedPerm<'db> {
    #[return_ref]
    pub chains: Vec<RedChain<'db>>,
}

impl<'db> Err<'db> for RedPerm<'db> {
    fn err(db: &'db dyn crate::Db, reported: Reported) -> Self {
        RedPerm::new(db, [RedChain::err(db, reported)])
    }
}

#[derive(SalsaSerialize)]
#[salsa::interned(debug)]
pub struct RedChain<'db> {
    #[return_ref]
    pub links: Vec<RedLink<'db>>,
}

impl<'db> RedChain<'db> {
    pub fn my(db: &'db dyn crate::Db) -> Self {
        RedChain::new(db, [])
    }

    pub fn our(db: &'db dyn crate::Db) -> Self {
        RedChain::new(db, [RedLink::Our])
    }
}

impl<'db> Err<'db> for RedChain<'db> {
    fn err(db: &'db dyn crate::Db, reported: Reported) -> Self {
        RedChain::new(db, [RedLink::Error(reported)])
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum RedLink<'db> {
    Our,
    RefLive(SymPlace<'db>),
    RefDead(SymPlace<'db>),
    MutLive(SymPlace<'db>),
    MutDead(SymPlace<'db>),
    Var(SymVariable<'db>),
    Error(Reported),
}

impl<'db> Err<'db> for RedLink<'db> {
    fn err(_db: &'db dyn crate::Db, reported: Reported) -> Self {
        RedLink::Error(reported)
    }
}

impl<'db> RedLink<'db> {
    pub fn is_copy(&self, env: &Env<'db>) -> Errors<bool> {
        match self {
            RedLink::Our => Ok(true),
            RedLink::RefLive(_) | RedLink::RefDead(_) => Ok(true),
            RedLink::MutLive(_) | RedLink::MutDead(_) => Ok(false),
            RedLink::Var(v) => Ok(env.var_is_declared_to_be(*v, Predicate::Copy)),
            RedLink::Error(reported) => Err(*reported),
        }
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
