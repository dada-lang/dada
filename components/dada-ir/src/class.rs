use crate::{parameter::Parameter, span::FileSpan, word::SpannedWord};

#[salsa::tracked]
pub struct Class {
    #[id]
    name: SpannedWord,

    #[return_ref]
    fields: Vec<Parameter>,

    /// Overall span of the class (including any body)
    span: FileSpan,
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _db: &Db) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
