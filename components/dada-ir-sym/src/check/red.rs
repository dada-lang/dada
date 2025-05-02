//! "Chains" are a canonicalized form of types/permissions.
//! They can only be produced after inference is complete as they require enumerating the bounds of inference variables.
//! They are used in borrow checking and for producing the final version of each inference variable.

use dada_ir_ast::diagnostic::{Err, Reported};
use dada_util::SalsaSerialize;
use salsa::Update;
use serde::Serialize;

use crate::ir::{
    indices::InferVarIndex,
    types::{SymGenericTerm, SymPlace, SymTyName},
    variables::SymVariable,
};

use super::{env::Env, predicates::Predicate};

mod glb;

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

impl<'db> RedPerm<'db> {
    pub fn is_provably(self, env: &Env<'db>, predicate: Predicate) -> bool {
        let chains = self.chains(env.db());
        assert!(!chains.is_empty());
        chains.iter().all(|chain| chain.is_provably(env, predicate))
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

    pub fn is_provably(self, env: &Env<'db>, predicate: Predicate) -> bool {
        let db = env.db();
        match predicate {
            Predicate::Copy => RedLink::are_copy(env, self.links(db)),
            Predicate::Move => RedLink::are_move(env, self.links(db)),
            Predicate::Owned => RedLink::are_owned(env, self.links(db)),
            Predicate::Lent => RedLink::are_lent(env, self.links(db)),
        }
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
}

impl<'db> RedLink<'db> {
    pub fn are_copy(env: &Env<'db>, links: &[Self]) -> bool {
        let Some(first) = links.first() else {
            return false;
        };
        first.is_copy(env)
    }

    pub fn are_move(env: &Env<'db>, links: &[Self]) -> bool {
        links.iter().all(|link| link.is_move(env))
    }

    pub fn are_owned(env: &Env<'db>, links: &[Self]) -> bool {
        links.iter().all(|link| link.is_owned(env))
    }

    pub fn are_lent(env: &Env<'db>, links: &[Self]) -> bool {
        links.iter().any(|link| link.is_lent(env))
    }

    pub fn is_my(&self, env: &Env<'db>) -> bool {
        match self {
            RedLink::Var(v) => {
                env.var_is_declared_to_be(*v, Predicate::Move)
                    && env.var_is_declared_to_be(*v, Predicate::Owned)
            }

            RedLink::Our
            | RedLink::RefLive(_)
            | RedLink::RefDead(_)
            | RedLink::MutLive(_)
            | RedLink::MutDead(_) => false,
        }
    }

    pub fn is_our(&self, env: &Env<'db>) -> bool {
        match self {
            RedLink::Our => true,
            RedLink::Var(v) => {
                env.var_is_declared_to_be(*v, Predicate::Copy)
                    && env.var_is_declared_to_be(*v, Predicate::Owned)
            }

            RedLink::RefLive(_)
            | RedLink::RefDead(_)
            | RedLink::MutLive(_)
            | RedLink::MutDead(_) => false,
        }
    }

    pub fn is_owned(&self, env: &Env<'db>) -> bool {
        match self {
            RedLink::Our => true,
            RedLink::Var(v) => env.var_is_declared_to_be(*v, Predicate::Owned),

            RedLink::RefLive(_)
            | RedLink::RefDead(_)
            | RedLink::MutLive(_)
            | RedLink::MutDead(_) => false,
        }
    }

    pub fn is_lent(&self, env: &Env<'db>) -> bool {
        match self {
            RedLink::RefLive(_)
            | RedLink::RefDead(_)
            | RedLink::MutLive(_)
            | RedLink::MutDead(_) => true,
            RedLink::Var(v) => env.var_is_declared_to_be(*v, Predicate::Lent),

            RedLink::Our => false,
        }
    }

    pub fn is_move(&self, env: &Env<'db>) -> bool {
        match self {
            RedLink::MutLive(_) | RedLink::MutDead(_) => true,
            RedLink::Var(v) => env.var_is_declared_to_be(*v, Predicate::Move),

            RedLink::Our | RedLink::RefLive(_) | RedLink::RefDead(_) => false,
        }
    }

    pub fn is_copy(&self, env: &Env<'db>) -> bool {
        match self {
            RedLink::Our | RedLink::RefLive(_) | RedLink::RefDead(_) => true,
            RedLink::Var(v) => env.var_is_declared_to_be(*v, Predicate::Copy),

            RedLink::MutLive(_) | RedLink::MutDead(_) => false,
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
