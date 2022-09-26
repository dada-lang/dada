//! Represents Dada types as they appear in signatures and things.
//! These interned values are produced by queries that read the syntax tree from the function declaration.
//! During type-checking, we use a different, richer representation that supports inference variables.

use salsa::DebugWithDb;

use crate::{
    class::Class,
    storage::Joint,
    word::{Word, Words},
};

/// Dada type appearing in a function signature. Types used during type checker
/// (which support inference) are different.
#[salsa::interned]
pub struct Ty {
    data: TyData,
}

/// Types can be generic parameters (`T`) or a specific class (`String`).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TyData {
    Parameter(ParameterTy),
    Class(ClassTy),
}

/// Dada type referencing a generic parameter
#[salsa::interned]
pub struct ParameterTy {
    /// Generic parameters have a permission; `T` on its own defaults to `my T`, but
    /// you might also write `shared T` or some such thing.
    pub permission: Permission,

    /// Name of the generic parameter
    pub name: Word,
}

/// Dada type referencing a specific class
#[salsa::interned]
pub struct ClassTy {
    /// Permissions used to access the object.
    pub permission: Permission,

    /// Class of the object (e.g., `String`).
    pub class: Class,

    /// Generic parameters (if any) to the class.
    pub generics: Tys,
}

/// A Dada *permission* from a signature.
#[salsa::interned]
pub struct Permission {
    data: PermissionData,
}

/// Permissions can either be a generic parameter or something fixed.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PermissionData {
    Parameter(Word),
    Known(KnownPermission),
}

// A dada *permission*
#[salsa::interned]
pub struct KnownPermission {
    pub joint: Joint,
    pub lessors: Paths,
}

/// List of paths like `a.b.c, d.e.f`
#[salsa::interned]
pub struct Paths {
    #[return_ref]
    elements: Vec<Path>,
}

/// List of types for generic argments.
#[salsa::interned]
pub struct Tys {
    #[return_ref]
    elements: Vec<Ty>,
}

/// A *Path* begins with a local variable and adds fields, e.g., `a.b.c`.
#[salsa::interned]
pub struct Path {
    variable_name: Word,
    field_names: Words,
}

impl DebugWithDb<dyn crate::Db + '_> for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        match self.data(db) {
            TyData::Class(v) => v.fmt(f, db),
            TyData::Parameter(v) => v.fmt(f, db),
        }
    }
}

impl DebugWithDb<dyn crate::Db + '_> for ParameterTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_tuple("ParameterTy")
            .field(&self.permission(db).debug(db))
            .field(&self.name(db).debug(db))
            .finish()
    }
}

impl DebugWithDb<dyn crate::Db + '_> for ClassTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_tuple("ClassTy")
            .field(&self.permission(db).debug(db))
            .field(&self.class(db).name(db).debug(db))
            .field(&self.generics(db).debug(db))
            .finish()
    }
}

impl DebugWithDb<dyn crate::Db + '_> for Paths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_tuple("Paths")
            .field(&self.elements(db).debug(db))
            .finish()
    }
}

impl DebugWithDb<dyn crate::Db + '_> for Tys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_tuple("Tys")
            .field(&self.elements(db).debug(db))
            .finish()
    }
}

impl DebugWithDb<dyn crate::Db + '_> for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_tuple("Path")
            .field(&self.variable_name(db).debug(db))
            .field(&self.field_names(db).debug(db))
            .finish()
    }
}

impl DebugWithDb<dyn crate::Db + '_> for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        match self.data(db) {
            PermissionData::Known(v) => v.fmt(f, db),
            PermissionData::Parameter(v) => v.fmt(f, db),
        }
    }
}

impl DebugWithDb<dyn crate::Db + '_> for KnownPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        f.debug_tuple("Permission")
            .field(&self.joint(db))
            .field(&self.lessors(db).debug(db))
            .finish()
    }
}
