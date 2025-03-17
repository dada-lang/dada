use std::str::FromStr;
use std::sync::{Arc, Mutex};

use dada_compiler::{Compiler, Fork, RealFs};
use dada_ir_ast::diagnostic::{Diagnostic, DiagnosticLabel, Level};
use dada_ir_ast::inputs::SourceFile;
use dada_ir_ast::span::{AbsoluteOffset, AbsoluteSpan};
use dada_util::{Fallible, Map, Set, bail};
use lsp::{Editor, Lsp, LspFork};
use lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, HoverProviderCapability, MessageType,
    OneOf, PublishDiagnosticsParams, TextDocumentContentChangeEvent, TextDocumentItem,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, Uri,
    VersionedTextDocumentIdentifier,
};
use lsp_types::{InitializeParams, ServerCapabilities};

use salsa::Setter;
use url::Url;

mod lsp;

fn main() -> Fallible<()> {
    Server::run()
}

struct Server {
    db: Compiler,
    diagnostics: Arc<Mutex<EditorDiagnostics>>,
}

/// Tracks the diagnostics we have sent over to the editor.
#[derive(Default)]
struct EditorDiagnostics {
    /// Track the source files for which we have published diagnostics to the editor.
    has_published_diagnostics: Set<SourceFile>,
}

impl lsp::Lsp for Server {
    type Fork = ServerFork;

    fn new(_params: InitializeParams) -> Fallible<Self> {
        Ok(Server {
            db: Compiler::new(RealFs::new(), None),
            diagnostics: Default::default(),
        })
    }

    fn server_capabilities(&mut self) -> Fallible<ServerCapabilities> {
        Ok(ServerCapabilities {
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    will_save: None,
                    will_save_wait_until: None,
                    save: None,
                },
            )),
            definition_provider: Some(OneOf::Left(true)),
            ..ServerCapabilities::default()
        })
    }

    fn server_info(&mut self) -> Fallible<Option<lsp_types::ServerInfo>> {
        Ok(None)
    }

    fn fork(&mut self) -> Self::Fork {
        ServerFork {
            db: self.db.fork(),
            diagnostics: self.diagnostics.clone(),
        }
    }

    fn did_open(
        &mut self,
        editor: &mut dyn Editor<Self>,
        params: DidOpenTextDocumentParams,
    ) -> Fallible<()> {
        let DidOpenTextDocumentParams { text_document } = params;
        let TextDocumentItem {
            uri,
            language_id: _,
            version: _,
            text,
        } = text_document;

        let source_file = self.db.open_source_file(uri.as_str(), Ok(text))?;

        editor.show_message(MessageType::INFO, format!("did open {}", uri.as_str()))?;

        editor.spawn(ServerFork::check_all_task(source_file));

        Ok(())
    }

    fn did_change(
        &mut self,
        editor: &mut dyn Editor<Self>,
        params: DidChangeTextDocumentParams,
    ) -> Fallible<()> {
        let DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version: _ },
            content_changes,
        } = params;
        let uri_str = uri.as_str();

        let source_file = self.db.get_previously_opened_source_file(uri_str)?;

        for TextDocumentContentChangeEvent {
            range,
            range_length: _,
            text,
        } in content_changes
        {
            match range {
                Some(_) => {
                    // FIXME: We should implement incremental change events; but LSP sends line/column
                    // positions and that's just *annoying*.
                    bail!("we requested full content change events");
                }
                None => {
                    let _old_contents = source_file.set_contents(&mut self.db).to(Ok(text));
                }
            }
        }

        editor.show_message(MessageType::INFO, format!("did change {uri_str}"))?;

        editor.spawn(ServerFork::check_all_task(source_file));

        Ok(())
    }
}

struct ServerFork {
    db: Fork<Compiler>,
    diagnostics: Arc<Mutex<EditorDiagnostics>>,
}

impl LspFork for ServerFork {
    fn fork(&self) -> Self {
        ServerFork {
            db: self.db.fork(),
            diagnostics: self.diagnostics.clone(),
        }
    }
}

type CheckAllTask = Box<dyn FnOnce(&ServerFork, &mut dyn Editor<Server>) -> Fallible<()> + Send>;

impl ServerFork {
    fn check_all_task(source_file: SourceFile) -> CheckAllTask {
        Box::new(move |this, editor| this.check_all(editor, source_file))
    }

    fn check_all(&self, editor: &mut dyn Editor<Server>, source_file: SourceFile) -> Fallible<()> {
        let new_diagnostics = self.db.check_all(source_file);
        self.diagnostics.lock().unwrap().reconcile_diagnostics(
            &self.db,
            editor,
            new_diagnostics,
        )?;
        Ok(())
    }
}

impl EditorDiagnostics {
    fn reconcile_diagnostics(
        &mut self,
        db: &Compiler,
        editor: &mut dyn Editor<Server>,
        diagnostics: Vec<Diagnostic>,
    ) -> Fallible<()> {
        let mut new_diagnostics: Map<SourceFile, Vec<Diagnostic>> = Map::default();

        // Sort diagnostics by URI
        for diagnostic in diagnostics {
            new_diagnostics
                .entry(diagnostic.span.source_file)
                .or_default()
                .push(diagnostic);
        }

        // Publish new diagnostics for each URI that has them
        for (&source_file, diagnostics) in &new_diagnostics {
            editor.publish_diagnostics(PublishDiagnosticsParams {
                uri: Self::lsp_uri(source_file.url(db)),
                diagnostics: diagnostics
                    .iter()
                    .map(|d| Self::lsp_diagnostic(db, d))
                    .collect(),
                version: None,
            })?;

            // Record that we successfully published diagnostics for this source file
            self.has_published_diagnostics.insert(source_file);
        }

        // Clear out diagnostics for URIs that no longer have any
        let no_longer_have_diagnostics: Vec<SourceFile> = self
            .has_published_diagnostics
            .iter()
            .filter(|source| !new_diagnostics.contains_key(source))
            .copied()
            .collect();
        for source_file in no_longer_have_diagnostics {
            editor.publish_diagnostics(PublishDiagnosticsParams {
                uri: Self::lsp_uri(source_file.url(db)),
                diagnostics: vec![],
                version: None,
            })?;
            self.has_published_diagnostics.remove(&source_file);
        }

        Ok(())
    }

    fn lsp_diagnostic(db: &Compiler, diagnostic: &Diagnostic) -> lsp_types::Diagnostic {
        let related_information = diagnostic
            .labels
            .iter()
            .map(|label| Self::lsp_diagnostic_related_information(db, label))
            .collect();

        lsp_types::Diagnostic {
            range: Self::lsp_range(db, diagnostic.span),
            severity: Some(Self::lsp_severity(db, diagnostic.level)),
            code: None,
            code_description: None,
            source: Some("Dada compiler".to_string()),
            message: diagnostic.message.clone(),
            related_information: Some(related_information),
            tags: None,
            data: None,
        }
    }

    fn lsp_severity(_db: &Compiler, level: Level) -> lsp_types::DiagnosticSeverity {
        match level {
            Level::Note => lsp_types::DiagnosticSeverity::INFORMATION,
            Level::Help => lsp_types::DiagnosticSeverity::HINT,
            Level::Info => lsp_types::DiagnosticSeverity::INFORMATION,
            Level::Warning => lsp_types::DiagnosticSeverity::WARNING,
            Level::Error => lsp_types::DiagnosticSeverity::ERROR,
        }
    }

    fn lsp_diagnostic_related_information(
        db: &Compiler,
        label: &DiagnosticLabel,
    ) -> lsp_types::DiagnosticRelatedInformation {
        let location = Self::lsp_location(db, label.span);
        let message = label.message.clone();
        lsp_types::DiagnosticRelatedInformation { location, message }
    }

    fn lsp_location(db: &Compiler, span: AbsoluteSpan) -> lsp_types::Location {
        let uri = Self::lsp_uri(span.source_file.url(db));
        let range = Self::lsp_range(db, span);
        lsp_types::Location { uri, range }
    }

    fn lsp_range(db: &Compiler, span: AbsoluteSpan) -> lsp_types::Range {
        let start = Self::lsp_position(db, span.source_file, span.start);
        let end = Self::lsp_position(db, span.source_file, span.end);
        lsp_types::Range { start, end }
    }

    fn lsp_position(
        db: &Compiler,
        source_file: SourceFile,
        offset: AbsoluteOffset,
    ) -> lsp_types::Position {
        let (line, column) = source_file.line_col(db, offset);
        lsp_types::Position {
            line: line.as_u32(),
            character: column.as_u32(),
        }
    }

    fn lsp_uri(url: &Url) -> Uri {
        Uri::from_str(url.as_str()).unwrap()
    }
}
