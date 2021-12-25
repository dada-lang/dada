use crate::{code::Code, parameter::UnparsedParameters, span::FileSpan, word::Word};

salsa::entity2! {
    entity Function in crate::Jar {
        #[id] name: Word,
        name_span: FileSpan,
        effect: Effect,
        unparsed_parameters: UnparsedParameters,
        code: Code,
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Effect {
    None,
    Async,
}

salsa::entity2! {
    entity Variable in crate::Jar {
        #[id] name: Word,
    }
}
