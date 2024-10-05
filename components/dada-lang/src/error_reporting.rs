use annotate_snippets::{Message, Renderer, Snippet};
use dada_ir_ast::{diagnostic::Diagnostic, span::AbsoluteSpan};

use crate::{db, GlobalOptions};

pub trait RenderDiagnostic {
    fn render(&self, opts: &GlobalOptions, db: &db::Database) -> String;
}

impl RenderDiagnostic for Diagnostic {
    fn render(&self, opts: &GlobalOptions, db: &db::Database) -> String {
        let message = to_message(db, self);
        renderer(opts).render(message).to_string()
    }
}

fn renderer(opts: &GlobalOptions) -> Renderer {
    if opts.no_color {
        Renderer::plain()
    } else {
        Renderer::styled()
    }
}

fn to_level(level: dada_ir_ast::diagnostic::Level) -> annotate_snippets::Level {
    match level {
        dada_ir_ast::diagnostic::Level::Note => annotate_snippets::Level::Note,
        dada_ir_ast::diagnostic::Level::Warning => annotate_snippets::Level::Warning,
        dada_ir_ast::diagnostic::Level::Info => annotate_snippets::Level::Info,
        dada_ir_ast::diagnostic::Level::Help => annotate_snippets::Level::Help,
        dada_ir_ast::diagnostic::Level::Error => annotate_snippets::Level::Error,
    }
}

fn to_message<'a>(db: &'a db::Database, diagnostic: &'a Diagnostic) -> Message<'a> {
    to_level(diagnostic.level)
        .title(&diagnostic.message)
        .snippet(to_snippet(db, diagnostic))
        .footers(diagnostic.children.iter().map(|d| to_message(db, d)))
}

fn to_snippet<'a>(db: &'a db::Database, diagnostic: &'a Diagnostic) -> Snippet<'a> {
    let source_file = diagnostic.span.source_file;

    let default_label = if !diagnostic.labels.is_empty() {
        None
    } else {
        Some(
            to_level(diagnostic.level)
                .span(to_span(diagnostic.span))
                .label("here"),
        )
    };

    Snippet::source(source_file.contents_if_ok(db))
        .line_start(1)
        .origin(source_file.path(db))
        .fold(true)
        .annotations(
            diagnostic
                .labels
                .iter()
                .map(|label| {
                    assert!(label.span.source_file == source_file);
                    to_level(label.level)
                        .span(to_span(label.span))
                        .label(&label.message)
                })
                .chain(default_label),
        )
}

fn to_span(span: AbsoluteSpan) -> std::ops::Range<usize> {
    span.start.as_usize()..span.end.as_usize()
}
