//! "Chains" are a canonicalized form of types/permissions.
//! They can only be produced after inference is complete as they require enumerating the bounds of inference variables.
//! They are used in borrow checking and for producing the final version of each inference variable.

use dada_ir_ast::diagnostic::{Err, Reported};
use dada_util::vecset::VecSet;
use salsa::Update;

use crate::ir::{
    indices::{FromInfer, InferVarIndex},
    types::{SymGenericTerm, SymPerm, SymPlace, SymTyName},
    variables::SymVariable,
};

/// A "red(uced) term" combines the possible permissions (a [`VecSet`] of [`Chain`])
/// with the type of the term (a [`RedTy`]). It can be used to represent either permissions or types.
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Update)]
pub struct RedTerm<'db> {
    pub chains: VecSet<Chain<'db>>,
    pub ty: RedTy<'db>,
}

impl<'db> RedTerm<'db> {
    /// Create a new [`RedTerm`].
    pub fn new(_db: &'db dyn crate::Db, chains: VecSet<Chain<'db>>, ty: RedTy<'db>) -> Self {
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
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Update)]
pub struct Chain<'db> {
    pub liens: Vec<Lien<'db>>,
}

impl<'db> Chain<'db> {
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

/// An individual unit in a [`Chain`][], representing a particular way of reaching data.
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
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Hash, Update)]
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

/// Captures the [`RedInfer`][] values for
/// a set of inference varaibles. Returned alongside
/// with the body of a
#[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Update)]
pub struct RedInfers<'db> {
    red_infers: Vec<RedInfer<'db>>,
}

impl<'db> RedInfers<'db> {
    pub fn new(red_infers: Vec<RedInfer<'db>>) -> Self {
        Self { red_infers }
    }

    /// Get the inferred value for variable `infer`
    pub fn red_infer(&self, infer: InferVarIndex) -> &RedInfer<'db> {
        &self.red_infers[infer.as_usize()]
    }

    /// True if there are no inference variables
    pub fn is_empty(&self) -> bool {
        self.red_infers.is_empty()
    }
}

/// The "reduced" value of an inference variable.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Update)]
pub enum RedInfer<'db> {
    Perm {
        lower: Vec<Chain<'db>>,
        upper: Vec<Chain<'db>>,
    },

    Ty {
        perm: InferVarIndex,
        red_ty: RedTy<'db>,
    },
}

/// Encodes whether a value is stored "flat" (by value)
/// or "pointer" (by reference).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update)]
pub enum Layout<'db> {
    /// By value -- my/our/shared
    Flat,

    /// By reference -- leased
    Pointer,

    /// Depends on the results of substitution
    Var(Vec<SymVariable<'db>>),
}
