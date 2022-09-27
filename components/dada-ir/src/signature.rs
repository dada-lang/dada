//! Represents Dada types as they appear in signatures and things.
//! These interned values are produced by queries that read the syntax tree from the function declaration.
//! During type-checking, we use a different, richer representation that supports inference variables.

use dada_id::id;
use derive_new::new;
use salsa::DebugWithDb;

use crate::{
    class::Class,
    storage::Atomic,
    word::{Word, Words},
};

#[salsa::tracked]
/// Represents a function parameter or a class field (which are declared in a parameter list).
pub struct Parameter {
    #[id]
    name: Word,

    /// Was the parameter/field represented with a type?
    ty: Option<Ty>,

    /// Was the parameter/field declared with atomic?
    /// (Only relevant to fields.)
    atomic: Atomic,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Signature {
    generics: Vec<GenericParameter>,
    where_clauses: Vec<WhereClause>,
    inputs: Vec<Ty>,
    output: Ty,
}

#[derive(new, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenericParameter {
    pub kind: GenericParameterKind,
    pub name: Option<Word>,
    pub index: ParameterIndex,
}

/// Types can be generic parameters (`T`) or a specific class (`String`).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GenericParameterKind {
    Permission,
    Type,
}

/// Types can be generic parameters (`T`) or a specific class (`String`).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WhereClause {
    IsShared(Permission),
    IsLeased(Permission),
}

/// Dada type appearing in a function signature. Types used during type checker
/// (which support inference) are different.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ty {
    /// Generic parameter type like `T`.
    Parameter(ParameterTy),

    /// Specific class like `String`.
    Class(ClassTy),

    /// A type that failed to validate in some way.
    /// The error will have already been reported to the user.
    Error,
}

/// Dada type referencing a generic parameter
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParameterTy {
    /// Generic parameters have a permission; `T` on its own defaults to `my T`, but
    /// you might also write `shared T` or some such thing.
    pub permission: Permission,

    /// Index of the generic parameter
    pub index: ParameterIndex,
}

id!(pub struct ParameterIndex);

/// Dada type referencing a specific class
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClassTy {
    /// Permissions used to access the object.
    pub permission: Permission,

    /// Class of the object (e.g., `String`).
    pub class: Class,

    /// Generic parameters (if any) to the class.
    pub generics: Vec<Ty>,
}

/// A Dada *permission* from a signature.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Permission {
    Parameter(ParameterIndex),
    Known(KnownPermission),
}

/// A dada *permission*, written like `shared{x, y, z}`.
/// `leased{x, y, z}` or  `given{x, y, z}`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KnownPermission {
    // The `shared`, `leased`, or `given`.
    pub kind: KnownPermissionKind,

    /// The `{x, y, z}` in the permission.
    pub paths: Vec<Path>,
}

/// Indicates how the value was derived from the given paths in a permission.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KnownPermissionKind {
    /// Data is *given* from the specified paths -- if those paths
    /// are owned permissions, then the result will be owned.
    /// If the set of paths is empty, this is a guaranteed `my` permission.
    Given,

    /// Data is *shared* from the specified paths -- if those paths
    /// are owned permissions, then the result will be owned.
    /// If the set of paths is empty, this is a guaranteed `our` permission.
    Shared,

    /// Data is *leased* from the specified paths, which cannot be an empty
    /// list. The user syntax `leased T` is translated to `P T` where `P: leased`.
    Leased,
}

/// A *Path* begins with a local variable and adds fields, e.g., `a.b.c`.
#[derive(new, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path {
    variable_name: Word,
    field_names: Words,
}

impl DebugWithDb<dyn crate::Db + '_> for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        match self {
            Ty::Class(v) => v.fmt(f, db),
            Ty::Parameter(v) => v.fmt(f, db),
            Ty::Error => f.debug_tuple("Error").finish(),
        }
    }
}

impl DebugWithDb<dyn crate::Db + '_> for ParameterTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_tuple("ParameterTy")
            .field(&self.permission.debug(db))
            .field(&self.index)
            .finish()
    }
}

impl DebugWithDb<dyn crate::Db + '_> for ClassTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_tuple("ClassTy")
            .field(&self.permission.debug(db))
            .field(&self.class.name(db).debug(db))
            .field(&self.generics.debug(db))
            .finish()
    }
}

impl DebugWithDb<dyn crate::Db + '_> for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_tuple("Path")
            .field(&self.variable_name.debug(db))
            .field(&self.field_names.debug(db))
            .finish()
    }
}

impl DebugWithDb<dyn crate::Db + '_> for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        match self {
            Permission::Known(v) => v.fmt(f, db),
            Permission::Parameter(v) => write!(f, "{:?}", v),
        }
    }
}

impl DebugWithDb<dyn crate::Db + '_> for KnownPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_tuple("Permission")
            .field(&self.kind)
            .field(&self.paths.debug(db))
            .finish()
    }
}
