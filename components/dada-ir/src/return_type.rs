use crate::span::FileSpan;

#[salsa::tracked]
/// Represents the return type of a function.
///
/// If `kind` is [ReturnTypeKind::Value] `span` is the span of `->`.
///
/// If `kind` is [ReturnTypeKind::Unit] `span` is the span between parameters and body.
pub struct ReturnType {
    kind: ReturnTypeKind,
    span: FileSpan,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ReturnTypeKind {
    Value,
    Unit,
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for ReturnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        let db = db.as_dyn_ir_db();
        write!(
            f,
            "ReturnType({:?}, {:?})",
            self.kind(db),
            self.span(db).into_debug(db)
        )
    }
}
