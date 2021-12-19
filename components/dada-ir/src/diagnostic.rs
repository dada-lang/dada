use crate::{span::Span, word::Word};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Diagnostic {
    pub filename: Word,
    pub span: Span,
    pub message: String,
}

#[salsa::accumulator(in crate::Jar)]
pub struct Diagnostics(Diagnostic);
