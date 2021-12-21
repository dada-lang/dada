use crossbeam_channel::Sender;
use dada_ir::{span::Offset, word::Word};
use lsp_server::Message;
use lsp_types::{
    notification::PublishDiagnostics, Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, Location, Position,
    PublishDiagnosticsParams, Range, Url,
};
use salsa::ParallelDatabase;

pub struct LspServerDatabase {
    db: dada_db::Db,
    threads: threadpool::ThreadPool,
    sender: Sender<Message>,
}

impl LspServerDatabase {
    pub fn new(sender: Sender<Message>) -> Self {
        Self {
            db: Default::default(),
            threads: Default::default(),
            sender,
        }
    }

    fn filename_from_uri(&self, uri: &Url) -> Word {
        let filename = uri.to_string();
        Word::from(&self.db, filename)
    }

    pub fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        let filename = self.filename_from_uri(&params.text_document.uri);
        let source_text = params.text_document.text;
        self.db.update_file(filename, source_text);
        self.spawn_check(
            params.text_document.uri,
            params.text_document.version,
            filename,
        );
    }

    pub fn did_change(&mut self, params: DidChangeTextDocumentParams) {
        let filename = self.filename_from_uri(&params.text_document.uri);
        // Since we asked for Sync full, just grab all the text from params
        let change = params.content_changes.into_iter().next().unwrap();
        let source_text = change.text;
        self.db.update_file(filename, source_text);
        self.spawn_check(
            params.text_document.uri,
            params.text_document.version,
            filename,
        );
    }

    fn spawn_check(&self, uri: Url, version: i32, filename: Word) {
        let sender = self.sender.clone();
        let db = self.db.snapshot();
        self.threads.execute(move || {
            let dada_diagnostics = db.diagnostics(filename);
            let diagnostics: Vec<_> = dada_diagnostics
                .into_iter()
                .map(|dada_diagnostic| db.lsp_diagnostic(dada_diagnostic))
                .collect();

            let diagnostic = PublishDiagnosticsParams {
                uri,
                diagnostics,
                version: Some(version),
            };

            let notification = super::new_notification::<PublishDiagnostics>(diagnostic);
            sender.send(Message::Notification(notification)).unwrap();
        });
    }
}

trait DadaLspMethods {
    fn lsp_position(&self, filename: Word, offset: Offset) -> Position;
    fn lsp_range(&self, span: dada_ir::span::FileSpan) -> Range;
    fn lsp_location(&self, span: dada_ir::span::FileSpan) -> Location;
    fn lsp_diagnostic(&self, dada_diagnostic: dada_ir::diagnostic::Diagnostic) -> Diagnostic;
}

impl DadaLspMethods for dada_db::Db {
    fn lsp_position(&self, filename: Word, offset: Offset) -> Position {
        let line_column = dada_lex::line_column(self, filename, offset);
        Position {
            line: line_column.line,
            character: line_column.column,
        }
    }

    fn lsp_range(&self, span: dada_ir::span::FileSpan) -> Range {
        Range {
            start: self.lsp_position(span.filename, span.start),
            end: self.lsp_position(span.filename, span.end),
        }
    }

    fn lsp_location(&self, span: dada_ir::span::FileSpan) -> Location {
        Location {
            uri: Url::parse(span.filename.as_str(self)).unwrap(),
            range: self.lsp_range(span),
        }
    }

    fn lsp_diagnostic(&self, dada_diagnostic: dada_ir::diagnostic::Diagnostic) -> Diagnostic {
        let range = self.lsp_range(dada_diagnostic.span);
        let severity = Some(match dada_diagnostic.severity {
            dada_ir::diagnostic::Severity::Help => DiagnosticSeverity::Hint,
            dada_ir::diagnostic::Severity::Note => DiagnosticSeverity::Information,
            dada_ir::diagnostic::Severity::Warning => DiagnosticSeverity::Warning,
            dada_ir::diagnostic::Severity::Error => DiagnosticSeverity::Error,
        });
        let code = None;
        let source = None;
        let message = dada_diagnostic.message.clone();
        let related_information = Some(
            dada_diagnostic
                .labels
                .into_iter()
                .map(|label| DiagnosticRelatedInformation {
                    location: self.lsp_location(label.span),
                    message: label.message,
                })
                .collect(),
        );
        let tags = None;
        Diagnostic {
            range,
            severity,
            code,
            source,
            message,
            related_information,
            tags,
        }
    }
}
