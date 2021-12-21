use crate::{span::Span, token_tree::TokenTree, word::Word};

/// Parse with methods from `dada_parse` prelude.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct UnparsedParameters(pub TokenTree);

salsa::entity2! {
    /// Represents a function parameter or a class field (which are declared in a parameter list).
    entity Parameter in crate::Jar {
        #[id] name: Word,
        mode: crate::storage_mode::StorageMode,
        ty: Option<crate::ty::Ty>,
        spans: ParameterSpans,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ParameterSpans {
    pub name: Span,
    pub mode: Span,
}
