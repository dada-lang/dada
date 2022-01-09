use crate::{span::FileSpan, token_tree::TokenTree, word::Word};

salsa::entity2! {
    entity Class in crate::Jar {
        #[id] name: Word,
        name_span: FileSpan,
        field_tokens: TokenTree,
    }
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _db: &Db) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
