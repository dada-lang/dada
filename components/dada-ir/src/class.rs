use crate::{parameter::UnparsedParameters, span::FileSpan, word::Word};

salsa::entity2! {
    entity Class in crate::Jar {
        #[id] name: Word,
        name_span: FileSpan,
        unparsed_parameters: UnparsedParameters,
    }
}

impl salsa::DebugWithDb<dyn crate::Db + '_> for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _db: &dyn crate::Db) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
