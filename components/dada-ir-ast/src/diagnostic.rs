use crate::{
    inputs::SourceFile,
    span::{AbsoluteOffset, Span},
};

#[salsa::accumulator]
pub struct Diagnostics(Diagnostic);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Diagnostic {
    source_file: SourceFile,
    start: AbsoluteOffset,
    end: AbsoluteOffset,
    message: String,
}

pub fn report_error<'db>(db: &'db dyn crate::Db, span: Span<'db>, message: String) {
    let (source_file, start, end) = span.absolute(db);
    Diagnostics::push(
        db,
        Diagnostic {
            source_file,
            start,
            end,
            message,
        },
    );
}
