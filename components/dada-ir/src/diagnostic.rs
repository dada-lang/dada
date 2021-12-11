use crate::{span::Span, word::Word};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Diagnostic {
    pub filename: Word,
    pub span: Span,
    pub message: String,
}
