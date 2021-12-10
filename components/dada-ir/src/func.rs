use crate::{code::Code, span::Span, word::Word};

salsa::entity2! {
    entity Func in crate::Jar {
        #[id] name: Word,
        effect: Effect,
        name_span: Span,
        arguments: Vec<Variable>,
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
