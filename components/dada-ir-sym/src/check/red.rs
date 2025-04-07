//! "Chains" are a canonicalized form of types/permissions.
//! They can only be produced after inference is complete as they require enumerating the bounds of inference variables.
//! They are used in borrow checking and for producing the final version of each inference variable.

use dada_ir_ast::diagnostic::{Err, Reported};
use dada_util::vecset::VecSet;
use salsa::Update;
use serde::Serialize;

use crate::ir::{
    indices::{FromInfer, InferVarIndex},
    types::{SymGenericTerm, SymPerm, SymPlace, SymTyName},
    variables::SymVariable,
};

/// A "red(uced) term" combines the possible permissions (a [`VecSet`] of [`Chain`])
/// with the type of the term (a [`RedTy`]). It can be used to represent either permissions or types.
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Update, Serialize)]
pub struct RedTerm<'db> {
    pub chains: VecSet<RedPerm<'db>>,
    pub ty: RedTy<'db>,
}

impl<'db> RedTerm<'db> {
    /// Create a new [`RedTerm`].
    pub fn new(_db: &'db dyn crate::Db, chains: VecSet<RedPerm<'db>>, ty: RedTy<'db>) -> Self {
        Self { ty, chains }
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
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Update, Serialize)]
pub struct RedPerm<'db> {
    pub liens: Vec<Lien<'db>>,
}

impl<'db> RedPerm<'db> {
    /// Create a new [`Chain`].
    pub fn new(_db: &'db dyn crate::Db, links: Vec<Lien<'db>>) -> Self {
        Self { liens: links }
    }

    pub fn from_head_tail(_db: &'db dyn crate::Db, head: Lien<'db>, tail: &[Lien<'db>]) -> Self {
        let mut liens = Vec::with_capacity(tail.len() + 1);
        liens.push(head);
        liens.extend(tail);
        Self { liens }
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
    pub fn shared(db: &'db dyn crate::Db, places: SymPlace<'db>) -> Self {
        Self::new(db, vec![Lien::Shared(places)])
    }

    /// Create a lien chain representing "leased from `place`".
    pub fn leased(db: &'db dyn crate::Db, places: SymPlace<'db>) -> Self {
        Self::new(db, vec![Lien::Leased(places)])
    }

    pub fn extend(&mut self, liens: &[Lien<'db>]) {
        self.liens.extend_from_slice(liens);
    }
}

impl<'db> std::ops::Deref for RedPerm<'db> {
    type Target = [Lien<'db>];

    fn deref(&self) -> &Self::Target {
        &self.liens
    }
}

impl<'db> Err<'db> for RedPerm<'db> {
    fn err(db: &'db dyn crate::Db, reported: Reported) -> Self {
        RedPerm::new(db, vec![Lien::Error(reported)])
    }
}

/// An individual unit in a [`Chain`][], representing a particular way of reaching data.
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Update, Serialize)]
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
