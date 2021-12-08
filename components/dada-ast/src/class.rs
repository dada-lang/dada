use crate::word::Word;

salsa::entity2! {
    entity Class in crate::Jar {
        #[id] name: Word,
        fields: Vec<Field>,
    }
}

salsa::entity2! {
    entity Field in crate::Jar {
        #[id] name: Word,
        mode: crate::storage_mode::StorageMode,
        ty: Option<crate::ty::Ty>,
    }
}
