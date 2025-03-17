use std::sync::atomic::{AtomicBool, Ordering};

use dada_util::Fallible;
use dispatch::LspDispatch;
use lsp_server::Connection;
use lsp_types::{
    InitializeParams, InitializeResult, PublishDiagnosticsParams, ServerCapabilities, ServerInfo,
    notification,
};

mod dispatch;

/// LSP server handlers.
pub trait Lsp: Sized {
    /// The server is "forked" to handle incoming "read" requests (e.g., goto-def).
    /// "Read" requests are requests that do not modify document state.
    type Fork: LspFork;

    fn run() -> Fallible<()> {
        run_server::<Self>()
    }

    fn new(params: InitializeParams) -> Fallible<Self>;

    /// Capabilities to report to the editor.
    fn server_capabilities(&mut self) -> Fallible<ServerCapabilities>;

    /// Server info to report to the editor.
    fn server_info(&mut self) -> Fallible<Option<ServerInfo>>;

    /// Create a "fork" of the LSP server that can be used from another thread.
    fn fork(&mut self) -> Self::Fork;

    /// Open reported for the given URI.
    fn did_open(
        &mut self,
        editor: &mut dyn Editor<Self>,
        item: lsp_types::DidOpenTextDocumentParams,
    ) -> Fallible<()>;

    /// Modification reported to the given URI.
    fn did_change(
        &mut self,
        editor: &mut dyn Editor<Self>,
        item: lsp_types::DidChangeTextDocumentParams,
    ) -> Fallible<()>;
}

pub trait LspFork: Sized + Send {
    #[expect(dead_code)]
    fn fork(&self) -> Self;
}

/// Allows your LSP server to make requests of the "editor".
///
/// The "editor" here includes the actual editor but also our dispatch loop,
/// which to you are not distinguishable.
pub trait Editor<L: Lsp> {
    /// Display a message to the user.
    fn show_message(
        &mut self,
        message_type: lsp_types::MessageType,
        message: String,
    ) -> Fallible<()>;

    fn publish_diagnostics(&mut self, params: PublishDiagnosticsParams) -> Fallible<()>;

    /// Enqueue a task to execute in parallel. The task may not start executing immediately.
    /// The task will be given a fork of the lsp along with an editor of its own.
    #[allow(clippy::type_complexity)]
    fn spawn(&mut self, task: Box<dyn FnOnce(&L::Fork, &mut dyn Editor<L>) -> Fallible<()> + Send>);
}

pub fn run_server<L: Lsp>() -> Fallible<()> {
    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    let lsp = initialize_server::<L>(&connection)?;

    LspDispatch::new(connection, lsp)
        .on_notification::<notification::DidOpenTextDocument>(Lsp::did_open)
        .on_notification::<notification::DidChangeTextDocument>(Lsp::did_change)
        .execute()?;

    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("shutting down server");
    Ok(())
}

static CANCEL: AtomicBool = AtomicBool::new(false);

fn not_canceled() -> bool {
    !CANCEL.load(Ordering::Relaxed)
}

fn initialize_server<L: Lsp>(connection: &Connection) -> Fallible<L> {
    let (initialize_id, initialize_params) = connection.initialize_start_while(not_canceled)?;
    let initialize_params: InitializeParams = serde_json::from_value(initialize_params)?;

    let mut server = L::new(initialize_params)?;

    let initialize_result = InitializeResult {
        capabilities: server.server_capabilities()?,
        server_info: server.server_info()?,
    };

    connection.initialize_finish_while(
        initialize_id,
        serde_json::to_value(initialize_result)?,
        not_canceled,
    )?;

    Ok(server)
}
