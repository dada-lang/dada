//! Defines the symbolic intermediate representation.
//! This is a type-checked, name-resolved version of the AST.
//! Also defines methods to create symbols (and the symbol tree) for functions, types, parameters, etc.

pub mod binder;
pub mod classes;
pub mod exprs;
pub mod functions;
pub mod indices;
pub mod module;
pub(crate) mod populate;
pub mod primitive;
pub mod subst;
pub mod types;
pub mod variables;
