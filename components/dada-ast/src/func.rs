use crate::{span::Span, word::Word};

salsa::entity2! {
    entity Func in crate::Jar {
        #[id] name: Word,
        effect: Effect,
        name_span: Span,
        arguments: Vec<Field>,
        body: FuncBody,
    }
}

pub enum Effect {
    None,
    Async,
}

salsa::entity2! {
    entity Variable in crate::Jar {
        #[id] name: Word,

    }
}
