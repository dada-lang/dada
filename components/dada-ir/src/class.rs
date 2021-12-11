use crate::{span::Span, token_tree::TokenTree, word::Word};

salsa::entity2! {
    entity Class in crate::Jar {
        #[id] name: Word,
        name_span: Span,
        field_tokens: TokenTree,
    }
}

salsa::entity2! {
    entity Field in crate::Jar {
        #[id] name: Word,
        name_span: Span,
        mode: crate::storage_mode::StorageMode,
        ty: Option<crate::ty::Ty>,
    }
}
