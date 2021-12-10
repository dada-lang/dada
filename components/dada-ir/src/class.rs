use crate::{span::Span, word::Word};

salsa::entity2! {
    entity Class in crate::Jar {
        #[id] name: Word,
        name_span: Span,
        fields: Vec<Field>,

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
