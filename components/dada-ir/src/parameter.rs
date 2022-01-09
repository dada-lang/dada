use crate::word::Word;

salsa::entity2! {
    /// Represents a function parameter or a class field (which are declared in a parameter list).
    entity Parameter in crate::Jar {
        #[id] name: Word,
        decl: crate::code::syntax::LocalVariableDeclData,
        decl_span: crate::code::syntax::LocalVariableDeclSpan,
    }
}
