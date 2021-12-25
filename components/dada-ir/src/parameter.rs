use crate::{token_tree::TokenTree, word::Word};

/// Parse with methods from `dada_parse` prelude.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct UnparsedParameters(pub TokenTree);

salsa::entity2! {
    /// Represents a function parameter or a class field (which are declared in a parameter list).
    entity Parameter in crate::Jar {
        #[id] name: Word,
        decl: crate::code::syntax::LocalVariableDeclData,
        spans: crate::code::syntax::LocalVariableDeclSpan,
        ty: Option<crate::ty::Ty>,
    }
}
