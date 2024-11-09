use crate::{diagnostic::Diagnostic, span::AbsoluteSpan};
use annotate_snippets::{Message, Renderer, Snippet};
use dada_util::arena::Arena;

use super::RenderOptions;

pub(super) fn render(db: &dyn crate::Db, opts: &RenderOptions, diagnostic: &Diagnostic) -> String {
    let arena = Arena::new();
    let message = to_message(db, diagnostic, &arena);
    let result = renderer(opts).render(message).to_string();
    result
}

fn renderer(opts: &RenderOptions) -> Renderer {
    if opts.no_color {
        Renderer::plain()
    } else {
        Renderer::styled()
    }
}

fn to_level(level: crate::diagnostic::Level) -> annotate_snippets::Level {
    match level {
        crate::diagnostic::Level::Note => annotate_snippets::Level::Note,
        crate::diagnostic::Level::Warning => annotate_snippets::Level::Warning,
        crate::diagnostic::Level::Info => annotate_snippets::Level::Info,
        crate::diagnostic::Level::Help => annotate_snippets::Level::Help,
        crate::diagnostic::Level::Error => annotate_snippets::Level::Error,
    }
}

fn to_message<'a>(
    db: &'a dyn crate::Db,
    diagnostic: &'a Diagnostic,
    arena: &'a Arena,
) -> Message<'a> {
    to_level(diagnostic.level)
        .title(&diagnostic.message)
        .snippet(to_snippet(db, diagnostic, arena))
        .footers(diagnostic.children.iter().map(|d| to_message(db, d, arena)))
}

fn to_snippet<'a>(
    db: &'a dyn crate::Db,
    diagnostic: &'a Diagnostic,
    arena: &'a Arena,
) -> Snippet<'a> {
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

    let url = source_file.url(db);
    let origin = arena.insert(db.url_display(&url));

    Snippet::source(source_file.contents_if_ok(db))
        .line_start(1)
        .origin(origin)
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
