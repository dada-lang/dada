use dada_ir::code::syntax::{Perm, Ty, TyData};

use super::{CodeParser, SpanFallover};

impl CodeParser<'_, '_> {
    pub(crate) fn parse_ty(&mut self) -> Option<Ty> {
        let perm = self.parse_perm();
        let path = self.parse_path();

        match (perm, path) {
            (None, None) => None,
            (Some(perm), None) => {
                self.error(
                    self.spans[perm],
                    "expected a path to follow this permission",
                )
                .emit(self.db);
                None
            }
            (perm, Some(path)) => Some(self.add(
                TyData { perm, path },
                self.span_consumed_since_parsing(perm.or_parsing(path)),
            )),
        }
    }

    pub(crate) fn parse_perm(&mut self) -> Option<Perm> {
        None
    }
}
