/// Side table that contains the spans for everything in an AST.
/// This isn't normally needed except for diagnostics, so it's
/// kept separate to avoid reducing incremental reuse.
/// You can request it by invoking the `spans`
/// method in the `dada_parse` prelude.
macro_rules! origin_table {
    ($(#[$attr:meta])* $pub:vis struct $table:ident { $($field:ident : $key:ty => $origins:ty,)* }) => {
        $(#[$attr])*
        $pub struct $table {
            $(
                $field: dada_collections::IndexVec<$key, $origins>,
            )*
        }

        impl<K> std::ops::Index<K> for $table
        where
            K: $crate::origin_table::HasOriginIn<$table>,
        {
            type Output = K::Origin;

            fn index(&self, index: K) -> &Self::Output {
                index.origin_in(self)
            }
        }

        impl $table {
            $pub fn get<K>(&self, k: K) -> K::Origin
            where
                K: $crate::origin_table::HasOriginIn<Self>,
            {
                <K::Origin>::clone(K::origin_in(k, self))
            }

            $pub fn push<K>(&mut self, k: K, s: K::Origin)
            where
                K: $crate::origin_table::PushOriginIn<Self>,
            {
                K::push_origin_in(k, self, s)
            }
        }

        $(
            impl $crate::origin_table::HasOriginIn<$table> for $key {
                type Origin = $origins;

                fn origin_in(self, table: &$table) -> &Self::Origin {
                    &table.$field[self]
                }
            }

            impl $crate::origin_table::PushOriginIn<$table> for $key {
                type Origin = $origins;

                fn push_origin_in(self, table: &mut $table, origin: Self::Origin) {
                    assert_eq!(<$key>::from(table.$field.len()), self);
                    table.$field.push(origin);
                }
            }
        )*
    }
}

pub trait HasOriginIn<T> {
    type Origin: Clone;

    fn origin_in(self, origins: &T) -> &Self::Origin;
}

pub trait PushOriginIn<T> {
    type Origin: Clone;

    fn push_origin_in(self, origins: &mut T, origin: Self::Origin);
}
