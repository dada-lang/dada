use crate::span::Span;

/// Side table that contains the spans for everything in an AST.
/// This isn't normally needed except for diagnostics, so it's
/// kept separate to avoid reducing incremental reuse.
/// You can request it by invoking the `spans`
/// method in the `dada_parse` prelude.
macro_rules! span_table {
    ($(#[$attr:meta])* $pub:vis struct $table:ident { $($field:ident : $key:ty => $spans:ty,)* }) => {
        $(#[$attr])*
        $pub struct $table {
            $(
                $field: dada_collections::IndexVec<$key, $spans>,
            )*
        }

        impl<K> std::ops::Index<K> for $table
        where
            K: $crate::span_table::HasSpanIn<$table>,
        {
            type Output = $crate::span::Span;

            fn index(&self, index: K) -> &Self::Output {
                EntireSpan::entire_span(index.span_in(self))
            }
        }

        impl $table {
            $pub fn get<K>(&self, k: K) -> K::Span
            where
                K: $crate::span_table::HasSpanIn<Self>,
            {
                <K::Span>::clone(K::span_in(k, self))
            }

            $pub fn push<K>(&mut self, k: K, s: K::Span)
            where
                K: $crate::span_table::PushSpanIn<Self>,
            {
                K::push_span_in(k, self, s)
            }
        }

        $(
            impl $crate::span_table::HasSpanIn<$table> for $key {
                type Span = $spans;

                fn span_in(self, table: &$table) -> &Self::Span {
                    &table.$field[self]
                }
            }

            impl $crate::span_table::PushSpanIn<$table> for $key {
                type Span = $spans;

                fn push_span_in(self, table: &mut $table, span: Self::Span) {
                    assert_eq!(<$key>::from(table.$field.len()), self);
                    table.$field.push(span);
                }
            }
        )*
    }
}

/// For some types, we want to track the span not just of the
/// entire thing but also of subentities. In that case, we map
/// the type to a struct with multiple spans. This trait
/// returns the span in that struct that represents the "entire thing",
/// which is what gets returned by some of the acccessor methods.
pub trait EntireSpan: 'static + Clone {
    fn entire_span(&self) -> &Span;
}

impl EntireSpan for Span {
    fn entire_span(&self) -> &Span {
        self
    }
}

pub trait HasSpanIn<T> {
    type Span: EntireSpan;

    fn span_in(self, spans: &T) -> &Self::Span;
}

pub trait PushSpanIn<T> {
    type Span: EntireSpan;

    fn push_span_in(self, spans: &mut T, span: Self::Span);
}
