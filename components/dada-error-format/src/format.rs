use std::io::Cursor;

use ariadne::{Config, Label, Report, ReportKind, Source};
use dada_ir::input_file::InputFile;

/// Options for controlling error formatting when they are printed.
#[derive(Clone, Copy)]
pub struct FormatOptions {
    /// Whether or not errors should use rich formatting with colors. This is generally turned on,
    /// except in tests, where the escape codes obscure the error messages.
    with_color: bool,
}

impl FormatOptions {
    pub fn no_color() -> Self {
        Self { with_color: false }
    }
}

const DEFAULT_FORMATTING: FormatOptions = FormatOptions { with_color: true };

pub fn print_diagnostic(
    db: &dyn crate::Db,
    diagnostic: &dada_ir::diagnostic::Diagnostic,
) -> eyre::Result<()> {
    Ok(ariadne_diagnostic(db, diagnostic, DEFAULT_FORMATTING)?.print(SourceCache::new(db))?)
}

pub fn format_diagnostics(
    db: &dyn crate::Db,
    diagnostics: &[dada_ir::diagnostic::Diagnostic],
) -> eyre::Result<String> {
    format_diagnostics_with_options(db, diagnostics, DEFAULT_FORMATTING)
}

pub fn format_diagnostics_with_options(
    db: &dyn crate::Db,
    diagnostics: &[dada_ir::diagnostic::Diagnostic],
    options: FormatOptions,
) -> eyre::Result<String> {
    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);
    let mut cache = SourceCache::new(db);
    for diagnostic in diagnostics {
        let ariadne = ariadne_diagnostic(db, diagnostic, options)?;
        ariadne.write(&mut cache, &mut cursor)?;
    }
    Ok(String::from_utf8(output)?)
}

fn ariadne_diagnostic(
    _db: &dyn crate::Db,
    diagnostic: &dada_ir::diagnostic::Diagnostic,
    options: FormatOptions,
) -> eyre::Result<ariadne::Report<ASpan>> {
    let mut builder = Report::<ASpan>::build(
        ReportKind::Error,
        diagnostic.span.input_file,
        diagnostic.span.start.into(),
    )
    .with_message(&diagnostic.message)
    .with_config(Config::default().with_color(options.with_color));

    for label in &diagnostic.labels {
        builder = builder.with_label(Label::new(ASpan(label.span())).with_message(&label.message));
    }

    Ok(builder.finish())
}

struct SourceCache<'me> {
    db: &'me dyn crate::Db,
    map: dada_collections::Map<InputFile, Source>,
}

impl<'me> SourceCache<'me> {
    pub fn new(db: &'me dyn crate::Db) -> Self {
        Self {
            db,
            map: Default::default(),
        }
    }
}

impl ariadne::Cache<InputFile> for SourceCache<'_> {
    fn fetch(&mut self, id: &InputFile) -> Result<&Source, Box<dyn std::fmt::Debug + '_>> {
        Ok(self.map.entry(*id).or_insert_with(|| {
            let source_text = id.source_text(self.db);
            Source::from(source_text)
        }))
    }

    fn display<'a>(&self, id: &'a InputFile) -> Option<Box<dyn std::fmt::Display + 'a>> {
        let s = id.name(self.db).as_str(self.db).to_string();
        Some(Box::new(s))
    }
}

struct ASpan(dada_ir::span::FileSpan);

impl ariadne::Span for ASpan {
    type SourceId = InputFile;

    fn source(&self) -> &Self::SourceId {
        &self.0.input_file
    }

    fn start(&self) -> usize {
        self.0.start.into()
    }

    fn end(&self) -> usize {
        self.0.end.into()
    }
}
