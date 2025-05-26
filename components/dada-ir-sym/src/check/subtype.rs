//! Subtyping relations and type conversions.
#![doc = include_str!("../../docs/subtyping.md")]

pub(crate) mod is_future;
pub(crate) mod is_numeric;
pub(crate) mod perms;
pub(crate) mod relate_infer_bounds;
pub(crate) mod terms;
