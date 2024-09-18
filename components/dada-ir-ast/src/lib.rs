#![allow(clippy::unused_unit)] // FIXME: salsa bug it seems

/// Macro to add `impl From<X> for Y` to enums.
/// Couldn't find a derive for this (!).
macro_rules! add_from_impls {
    ($(#[$attr:meta])* $v:vis enum $name:ident<$lt:lifetime> { $(
        $(#[$variant_meta:meta])*
        $variant:ident($variant_ty:ty),)*
    }) => {
        $(#[$attr])*
        $v enum $name<$lt> {
            $(
                $(#[$variant_meta])*
                $variant($variant_ty),
            )*
        }

        $(
            impl<$lt> From<$variant_ty> for $name<$lt> {
                fn from(v: $variant_ty) -> Self {
                    $name::$variant(v)
                }
            }
        )*
    };
}

pub mod ast;
pub mod diagnostic;
pub mod inputs;
pub mod span;

use salsa::Database as Db;
