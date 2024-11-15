use dada_compiler::{Compiler, Fork, RealFs};
use dada_util::{bail, Fallible};
use lsp::{Editor, Lsp, LspFork};
use lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, HoverProviderCapability, MessageType,
    OneOf, TextDocumentContentChangeEvent, TextDocumentItem, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions, VersionedTextDocumentIdentifier,
};
use lsp_types::{InitializeParams, ServerCapabilities};

use salsa::Setter;

mod lsp;

fn main() -> Fallible<()> {
    Server::run()
}

struct Server {
    compiler: Compiler,
}

impl lsp::Lsp for Server {
    type Fork = ServerFork;

    fn new(_params: InitializeParams) -> Fallible<Self> {
        Ok(Server {
            compiler: Compiler::new(RealFs::new()),
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
            compiler: self.compiler.fork(),
        }
    }

    fn did_open(&mut self, editor: &dyn Editor, params: DidOpenTextDocumentParams) -> Fallible<()> {
        let DidOpenTextDocumentParams { text_document } = params;
        let TextDocumentItem {
            uri,
            language_id: _,
            version: _,
            text,
        } = text_document;

        self.compiler.open_source_file(uri.as_str(), Ok(text))?;

        editor.show_message(MessageType::INFO, format!("did open {}", uri.as_str()))?;

        Ok(())
    }

    fn did_change(
        &mut self,
        editor: &dyn Editor,
        params: DidChangeTextDocumentParams,
    ) -> Fallible<()> {
        let DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version: _ },
            content_changes,
        } = params;
        let uri_str = uri.as_str();

        let source_file = self.compiler.get_previously_opened_source_file(uri_str)?;

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
                    let _old_contents = source_file.set_contents(&mut self.compiler).to(Ok(text));
                }
            }
        }

        editor.show_message(MessageType::INFO, format!("did change {uri_str}"))?;

        Ok(())
    }
}

struct ServerFork {
    #[expect(dead_code)]
    compiler: Fork<Compiler>,
}

impl LspFork for ServerFork {}
