use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use dada_util::Fallible;
use dispatch::LspDispatch;
use lsp_server::{Connection, Message};
use lsp_types::{notification, InitializeParams, InitializeResult, ServerCapabilities, ServerInfo};

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

    fn server_capabilities(&mut self) -> Fallible<ServerCapabilities>;
    fn server_info(&mut self) -> Fallible<Option<ServerInfo>>;

    #[expect(dead_code)]
    fn fork(&mut self) -> Self::Fork;

    fn did_open(
        &mut self,
        editor: &dyn Editor,
        item: lsp_types::DidOpenTextDocumentParams,
    ) -> Fallible<()>;
    fn did_change(
        &mut self,
        editor: &dyn Editor,
        item: lsp_types::DidChangeTextDocumentParams,
    ) -> Fallible<()>;
}

#[expect(dead_code)]
pub trait LspFork: Sized {}

pub trait Editor {
    fn show_message(&self, message_type: lsp_types::MessageType, message: String) -> Fallible<()>;
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
