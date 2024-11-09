use std::ops::ControlFlow;
use std::time::Duration;

use async_lsp::client_monitor::ClientProcessMonitorLayer;
use async_lsp::concurrency::ConcurrencyLayer;
use async_lsp::lsp_types::{
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
    HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, MarkedString,
    MessageType, OneOf, ServerCapabilities, ShowMessageParams, TextDocumentContentChangeEvent,
    TextDocumentItem, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    VersionedTextDocumentIdentifier,
};
use async_lsp::panic::CatchUnwindLayer;
use async_lsp::router::Router;
use async_lsp::server::LifecycleLayer;
use async_lsp::tracing::TracingLayer;
use async_lsp::{ClientSocket, ErrorCode, LanguageClient, LanguageServer, ResponseError};
use dada_compiler::{Compiler, RealFs};
use dada_util::anyhow;
use futures::future::BoxFuture;
use salsa::Setter;
use tower::ServiceBuilder;
use tracing::{info, Level};

macro_rules! try_or_return_lsp_error {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                return ControlFlow::Break(Err(async_lsp::Error::Response(ResponseError::new(
                    ErrorCode::REQUEST_FAILED,
                    e.to_string(),
                ))));
            }
        }
    };
}

struct ServerState {
    compiler: Compiler,
    client: ClientSocket,
    counter: i32,
}

impl LanguageServer for ServerState {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<async_lsp::Result<()>>;

    fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> BoxFuture<'static, Result<InitializeResult, Self::Error>> {
        eprintln!("Initialize with {params:?}");
        Box::pin(async move {
            Ok(InitializeResult {
                capabilities: ServerCapabilities {
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
                },
                server_info: None,
            })
        })
    }

    fn hover(&mut self, _: HoverParams) -> BoxFuture<'static, Result<Option<Hover>, Self::Error>> {
        let mut client = self.client.clone();
        let counter = self.counter;
        Box::pin(async move {
            client
                .show_message(ShowMessageParams {
                    typ: MessageType::INFO,
                    message: "Hello LSP".into(),
                })
                .unwrap();
            Ok(Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(format!(
                    "I am a hover text {counter}!"
                ))),
                range: None,
            }))
        })
    }

    fn definition(
        &mut self,
        _: GotoDefinitionParams,
    ) -> BoxFuture<'static, Result<Option<GotoDefinitionResponse>, ResponseError>> {
        unimplemented!("Not yet implemented!");
    }

    fn did_change_configuration(
        &mut self,
        _: DidChangeConfigurationParams,
    ) -> ControlFlow<async_lsp::Result<()>> {
        ControlFlow::Continue(())
    }

    #[must_use]
    fn did_open(&mut self, params: DidOpenTextDocumentParams) -> Self::NotifyResult {
        let DidOpenTextDocumentParams { text_document } = params;
        let TextDocumentItem {
            uri,
            language_id: _,
            version: _,
            text,
        } = text_document;

        try_or_return_lsp_error!(self.compiler.open_source_file(&uri, Ok(text)));

        ControlFlow::Continue(())
    }

    #[must_use]
    fn did_change(&mut self, params: DidChangeTextDocumentParams) -> Self::NotifyResult {
        let DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version: _ },
            content_changes,
        } = params;

        let source_file =
            try_or_return_lsp_error!(self.compiler.get_previously_opened_source_file(&uri));

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
                    try_or_return_lsp_error!(Err(anyhow!(
                        "we requested full content change events"
                    )));
                    unreachable!();
                }
                None => {
                    let _old_contents = source_file.set_contents(&mut self.compiler).to(Ok(text));
                }
            }
        }

        ControlFlow::Continue(())
    }

    #[must_use]
    fn did_close(&mut self, params: DidCloseTextDocumentParams) -> Self::NotifyResult {
        let DidCloseTextDocumentParams { text_document: _ } = params;

        // We don't really care when something is closed, do we?
        //
        // I guess there is a kind of "GC" that could occur when something is closed,
        // where we are able to unload non-opened source files from memory.

        ControlFlow::Continue(())
    }
}

struct TickEvent;

impl ServerState {
    fn new_router(client: ClientSocket) -> Router<Self> {
        let compiler = Compiler::new(RealFs::default());
        let mut router = Router::from_language_server(Self {
            compiler,
            client,
            counter: 0,
        });
        router.event(Self::on_tick);
        router
    }

    fn on_tick(&mut self, _: TickEvent) -> ControlFlow<async_lsp::Result<()>> {
        info!("tick");
        self.counter += 1;
        ControlFlow::Continue(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (server, _) = async_lsp::MainLoop::new_server(|client| {
        tokio::spawn({
            let client = client.clone();
            async move {
                let mut interval = tokio::time::interval(Duration::from_secs(1));
                loop {
                    interval.tick().await;
                    if client.emit(TickEvent).is_err() {
                        break;
                    }
                }
            }
        });

        ServiceBuilder::new()
            .layer(TracingLayer::default())
            .layer(LifecycleLayer::default())
            .layer(CatchUnwindLayer::default())
            .layer(ConcurrencyLayer::default())
            .layer(ClientProcessMonitorLayer::new(client.clone()))
            .service(ServerState::new_router(client))
    });

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .init();

    let (stdin, stdout) = (
        tokio_util::compat::TokioAsyncReadCompatExt::compat(tokio::io::stdin()),
        tokio_util::compat::TokioAsyncWriteCompatExt::compat_write(tokio::io::stdout()),
    );

    server.run_buffered(stdin, stdout).await.unwrap();
}
