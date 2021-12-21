use crate::{parameter::UnparsedParameters, span::Span, word::Word};

salsa::entity2! {
    entity Class in crate::Jar {
        #[id] name: Word,
        name_span: Span,
        unparsed_parameters: UnparsedParameters,
    }
}
