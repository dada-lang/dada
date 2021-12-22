//! The `dada_id` crate contains utilities for creating "local" ids that are specific to
//! a particular data structure, function, etc. These serve a similar role to salsa
//! interning and entities, but they are meant to be used for "internal" items (for example,
//! within a function, tracking the tree of expressions).
//!
//! In general each id I maps to some value V, but ids come in two forms:
//!
//! * allocating ids -- each time you add a value V, you get a fresh id I back.
//!   This is appropriate when you will be adding other "metadata" attached to the id,
//!   such as a span.
//! * interning ids -- if you add the same value V twice, you get back the same id I twice.
//!
//! To use these utilities, you make use of two macros:
//!
//! * `id!(pub struct Id)` creates a struct `Id` that can be used as an id.
//! * `tables! { .. }` declares a struct housing a set of `Id -> Value` mappings;
//!   also defines whether those are *allocating* or *interning* mappings.

use std::hash::Hash;

pub mod alloc_table;
pub mod intern_table;
pub mod prelude {
    pub use crate::InternKey;
    pub use crate::InternValue;
}

/// This module is used by the `tables` macro.
pub mod table_types {
    #![allow(non_camel_case_types)]
    pub type alloc<K, V> = crate::alloc_table::AllocTable<K, V>;
    pub type intern<K, V> = crate::intern_table::InternTable<K, V>;
}

/// Declares a struct usable as an id within a table.
#[macro_export]
macro_rules! id {
    ($v:vis struct $n:ident) => {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
        $v struct $n(salsa::Id);

        impl salsa::AsId for $n {
            fn as_id(self) -> salsa::Id {
                self.0
            }

            fn from_id(id: salsa::Id) -> Self {
                Self(id)
            }
        }

        impl From<usize> for $n {
            fn from(u: usize) -> $n {
                $n(salsa::Id::from(u))
            }
        }

        impl From<$n> for usize {
            fn from(n: $n) -> usize {
                n.0.into()
            }
        }
    }
}

/// Declares a struct containing a group of alloc/interning tables, along with methods for accessing them.
///
/// Example:
///
/// ```rust
/// tables! {
///     pub struct Foo {
///         exprs: alloc Expr => ExprData,
///         tys: intern Ty => TyData,
///     }
/// }
/// ```
#[macro_export]
macro_rules! tables {
    ($(#[$attr:meta])* $vis:vis struct $n:ident {
        $(
            $f:ident: $tty:ident $k:ty => $v:ty,
        )*
    }) => {
        $(#[$attr])*
        $vis struct $n {
            $(
                $f: dada_id::table_types::$tty<$k, $v>,
            )*
        }

        impl Default for $n {
            fn default() -> Self {
                Self {
                    $($f: <dada_id::table_types::$tty<$k,$v>>::default(),)*
                }
            }
        }

        impl<K> std::ops::Index<K> for $n
        where
            K: $crate::InternKey<Table = Self>,
        {
            type Output = K::Value;

            fn index(&self, key: K) -> &Self::Output {
                K::data(key, self)
            }
        }

        impl $n {
            pub fn add<V>(&mut self, value: V) -> V::Key
            where
                V: $crate::InternValue<Table = Self>,
            {
                dada_id::InternValue::add(value, self)
            }
        }

        $(
            impl $crate::InternValue for $v {
                type Table = $n;
                type Key = $k;

                fn add(self, table: &mut Self::Table) -> Self::Key {
                    table.$f.add(self)
                }
            }

            impl dada_id::InternKey for $k {
                type Table = $n;
                type Value = $v;

                fn data(self, table: &Self::Table) -> &Self::Value {
                    table.$f.data(self)
                }
            }
        )*
    }
}

pub trait InternValue: Hash + Eq {
    type Table;
    type Key: salsa::AsId;

    fn add(self, table: &mut Self::Table) -> Self::Key;
}

pub trait InternKey: salsa::AsId + 'static {
    type Table;
    type Value: Hash + Eq;

    fn data(self, table: &Self::Table) -> &Self::Value;
}
