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

/// Represents the fields of a class
#[derive(new, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClassStructure {
    /// Generics declared on the class. These introduce type parameters
    /// that will be referenced by the fields.
    pub generics: Vec<GenericParameter>,

    /// Where clauses declared on the class.
    pub where_clauses: Vec<WhereClause>,

    /// Fields declared in the class. Currently each of these fields is
    /// also declared in the class signature, but eventually we expect
    /// there to be add'l fields in a class declaration.
    pub fields: Vec<Field>,
}

impl ClassStructure {
    pub fn field_atomic(&self, index: usize) -> Atomic {
        self.fields[index].atomic
    }

    pub fn field_name(&self, index: usize) -> Word {
        self.fields[index].name
    }
}

/// A field in a class
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Field {
    /// Field name.
    pub name: Word,

    /// Type of the field.
    ///
    /// If None, ty was not given by user (dynamically typed).
    /// Note that this sort of wildcard "any" type can only occur at the top-level (by design).
    pub ty: Option<Ty>,

    /// Was the field declared as atomic?
    pub atomic: Atomic,
}

/// Represents the signature in a callable thing (e.g., a class, function, etc)
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Signature {
    /// Generics declared on the function. These introduce type parameters
    /// that will be referenced by the input/output types.
    pub generics: Vec<GenericParameter>,

    /// Where clauses that must be satisfied when the function is called.
    pub where_clauses: Vec<WhereClause>,

    /// Type and label of each function parameter
    pub inputs: Vec<InputTy>,

    /// Output or return type
    pub output: Option<Ty>,
}

/// Combine the label/type of a function parameter
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputTy {
    /// Parameter name.
    pub name: Word,

    /// If None, ty was not given by user (dynamically typed).
    /// Note that this sort of wildcard "any" type can only occur at the top-level (by design).
    pub ty: Option<Ty>,
}

/// Generic parameter declared on a class or function
#[derive(new, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenericParameter {
    /// The *kind* of a generic parameter indicates whether it is a type or permission
    pub kind: GenericParameterKind,

    /// If declared by the user, then `Some` with the name they gave it.
    /// If added by desugaring, then `None`.
    pub name: Option<Word>,

    /// Index of this parameter in the list of parameters. When this parameter is referenced,
    /// it will be referenced by this index.
    pub index: ParameterIndex,
}

/// Types can be generic parameters (`T`) or a specific class (`String`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GenericParameterKind {
    Permission,
    Type,
}

/// Where-clauses that can be attached to a signature
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WhereClause {
    IsShared(ParameterIndex),
    IsLeased(ParameterIndex),
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
    pub variable_name: Word,
    pub field_names: Words,
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
            Permission::Parameter(v) => write!(f, "{v:?}"),
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
