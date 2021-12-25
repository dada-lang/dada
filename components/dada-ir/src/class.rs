use crate::{parameter::UnparsedParameters, span::FileSpan, word::Word};

salsa::entity2! {
    entity Class in crate::Jar {
        #[id] name: Word,
        name_span: FileSpan,
        unparsed_parameters: UnparsedParameters,
    }
}
