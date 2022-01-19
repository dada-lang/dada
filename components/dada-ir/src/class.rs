use crate::{span::FileSpan, token_tree::TokenTree, word::Word};

salsa::entity2! {
    entity Class in crate::Jar {
        #[id] name: Word,
        field_tokens: TokenTree,

        /// Overall span of the class (including any body)
        span: FileSpan,

        /// Span of the class name specifically
        name_span: FileSpan,
    }
}

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _db: &Db) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
