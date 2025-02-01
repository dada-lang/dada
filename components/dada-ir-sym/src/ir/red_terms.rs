use dada_util::vecset::VecSet;
use salsa::Update;

use super::{
    types::{SymGenericTerm, SymPlace, SymTyName},
    variables::SymVariable,
};

/// "Red(uced) terms" are derived from user terms
/// and represent the final, reduced form of a permission or type.
/// No matter their kind, all terms are reduced to a [`RedPerms`][] and a [`RedTy`][],
/// with permission parameters being represented
/// using [`RedTy::None`][].
#[salsa::interned]
pub struct RedTerm<'db> {
    perm: RedPerm<'db>,
    ty: RedTy<'db>,
}

/// "Red(uced) perms" are derived from permission terms
/// written by users. They indicate the precise implications
/// of a permission. Many distinct permission terms can
/// be reduced to the same [`RedPerm`][]. For example:
///
/// * `leased[d1] our` and `our` are equivalent;
/// * `leased[d1] leased[d2]` and `leased[d1, d2]` are equivalent;
/// * and so forth.
///
/// In thinking about red-perms it is helpful to remember
/// the permission matrix:
///
/// |         | `move`       | `copy`                          |
/// |---------|--------------|---------------------------------|
/// | `owned` | `my`         | `our`                           |
/// | `lent`  | `leased[..]` | `shared[..]`,  `our leased[..]` |
///
/// All red perms represent something in this matrix (modulo generics).
#[salsa::interned]
pub struct RedPerm<'db> {
    #[return_ref]
    shared: VecSet<SymPlace<'db>>,

    #[return_ref]
    leased: VecSet<SymPlace<'db>>,

    #[return_ref]
    vars: VecSet<SymVariable<'db>>,
}

/// "Red(uced) types" are derived from user type terms
/// and represent the core type of the underlying value.
/// They represent only the type itself and not the permissions
/// on that type-- the full info is captured in the [`RedTerm`][]
/// that is created from the type. Another wrinkle is that [`RedTy`][]
/// values can be created from any generic term, including permissions,
/// in which case the [`RedTy`] variant is [`RedTy::None`].
#[derive(Clone, Hash, Update, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum RedTy<'db> {
    Var(SymVariable<'db>),
    Named(SymTyName<'db>, Vec<SymGenericTerm<'db>>),
    None,
}
