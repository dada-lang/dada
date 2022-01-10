use std::io::Cursor;

use ariadne::{Label, Report, ReportKind, Source};
use dada_ir::filename::Filename;

pub fn print_diagnostic(
    db: &dyn crate::Db,
    diagnostic: &dada_ir::diagnostic::Diagnostic,
) -> eyre::Result<()> {
    Ok(ariadne_diagnostic(db, diagnostic)?.print(SourceCache::new(db))?)
}

pub fn format_diagnostics(
    db: &dyn crate::Db,
    diagnostics: &[dada_ir::diagnostic::Diagnostic],
) -> eyre::Result<String> {
    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);
    let mut cache = SourceCache::new(db);
    for diagnostic in diagnostics {
        let ariadne = ariadne_diagnostic(db, diagnostic)?;
        ariadne.write(&mut cache, &mut cursor)?;
    }
    Ok(String::from_utf8(output)?)
}

fn ariadne_diagnostic(
    _db: &dyn crate::Db,
    diagnostic: &dada_ir::diagnostic::Diagnostic,
) -> eyre::Result<ariadne::Report<ASpan>> {
    let mut builder = Report::<ASpan>::build(
        ReportKind::Error,
        diagnostic.span.filename,
        diagnostic.span.start.into(),
    )
    .with_message(&diagnostic.message);

    for label in &diagnostic.labels {
        builder = builder.with_label(Label::new(ASpan(label.span())).with_message(&label.message));
    }

    Ok(builder.finish())
}

struct SourceCache<'me> {
    db: &'me dyn crate::Db,
    map: dada_collections::Map<Filename, Source>,
}

impl<'me> SourceCache<'me> {
    pub fn new(db: &'me dyn crate::Db) -> Self {
        Self {
            db,
            map: Default::default(),
        }
    }
}

impl ariadne::Cache<Filename> for SourceCache<'_> {
    fn fetch(&mut self, id: &Filename) -> Result<&Source, Box<dyn std::fmt::Debug + '_>> {
        Ok(self.map.entry(*id).or_insert_with(|| {
            let source_text = dada_ir::manifest::source_text(self.db, *id);
            Source::from(source_text)
        }))
    }

    fn display<'a>(&self, id: &'a Filename) -> Option<Box<dyn std::fmt::Display + 'a>> {
        let s = id.as_str(self.db).to_string();
        Some(Box::new(s))
    }
}

struct ASpan(dada_ir::span::FileSpan);

impl ariadne::Span for ASpan {
    type SourceId = Filename;

    fn source(&self) -> &Self::SourceId {
        &self.0.filename
    }

    fn start(&self) -> usize {
        self.0.start.into()
    }

    fn end(&self) -> usize {
        self.0.end.into()
    }
}
