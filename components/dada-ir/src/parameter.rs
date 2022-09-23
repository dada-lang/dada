use crate::{storage::Atomic, word::Word};

#[salsa::tracked]
/// Represents a function parameter or a class field (which are declared in a parameter list).
pub struct Parameter {
    #[id]
    name: Word,

    atomic: Atomic,
}
