use crate::{code::Code, span::Span, token_tree::TokenTree, word::Word};

salsa::entity2! {
    entity Function in crate::Jar {
        #[id] name: Word,
        name_span: Span,
        effect: Effect,
        argument_tokens: TokenTree,
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
