use crossbeam_channel::Sender;
use dada_collections::Map;
use dada_ir::{input_file::InputFile, span::Offset};
use lsp_server::Message;
use lsp_types::{
    notification::PublishDiagnostics, Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, Location, Position,
    PublishDiagnosticsParams, Range, Url,
};
use salsa::ParallelDatabase;

pub struct LspServerDatabase {
    db: dada_db::Db,
    input_files: Map<Url, InputFile>,
    threads: threadpool::ThreadPool,
    sender: Sender<Message>,
}

impl LspServerDatabase {
    pub fn new(sender: Sender<Message>) -> Self {
        Self {
            db: Default::default(),
            threads: Default::default(),
            input_files: Default::default(),
            sender,
        }
    }

    fn input_file_from_uri(&mut self, uri: &Url) -> InputFile {
        *self
            .input_files
            .entry(uri.clone())
            .or_insert_with(|| self.db.new_input_file(uri.to_string(), "".to_string()))
    }

    pub fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        let input_file = self.input_file_from_uri(&params.text_document.uri);
        let source_text = params.text_document.text;
        input_file.set_source_text(&mut self.db, source_text);
        self.spawn_check(
            params.text_document.uri,
            params.text_document.version,
            input_file,
        );
    }

    pub fn did_change(&mut self, params: DidChangeTextDocumentParams) {
        let input_file = self.input_file_from_uri(&params.text_document.uri);
        // Since we asked for Sync full, just grab all the text from params
        let change = params.content_changes.into_iter().next().unwrap();
        let source_text = change.text;
        input_file.set_source_text(&mut self.db, source_text);
        self.spawn_check(
            params.text_document.uri,
            params.text_document.version,
            input_file,
        );
    }

    fn spawn_check(&self, uri: Url, version: i32, input_file: InputFile) {
        let sender = self.sender.clone();
        let db = self.db.snapshot();
        self.threads.execute(move || {
            let dada_diagnostics = db.diagnostics(input_file);
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
    fn lsp_position(&self, input_file: InputFile, offset: Offset) -> Position;
    fn lsp_range(&self, span: dada_ir::span::FileSpan) -> Range;
    fn lsp_location(&self, span: dada_ir::span::FileSpan) -> Location;
    fn lsp_diagnostic(&self, dada_diagnostic: dada_ir::diagnostic::Diagnostic) -> Diagnostic;
}

impl DadaLspMethods for dada_db::Db {
    fn lsp_position(&self, input_file: InputFile, offset: Offset) -> Position {
        let line_column = dada_ir::lines::line_column(self, input_file, offset);
        Position {
            line: line_column.line1(),
            character: line_column.column1(),
        }
    }

    fn lsp_range(&self, span: dada_ir::span::FileSpan) -> Range {
        Range {
            start: self.lsp_position(span.input_file, span.start),
            end: self.lsp_position(span.input_file, span.end),
        }
    }

    fn lsp_location(&self, span: dada_ir::span::FileSpan) -> Location {
        Location {
            uri: Url::parse(span.input_file.name(self).string(self)).unwrap(),
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
