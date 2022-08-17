use db::LspServerDatabase;
use lsp_types::{
    notification::{DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument},
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
};
use serde::de::DeserializeOwned;

use lsp_server::{Connection, IoThreads, Message, Notification};

mod db;

pub struct LspServer {
    connection: Connection,
    #[allow(dead_code)]
    io_threads: IoThreads,
    db: db::LspServerDatabase,
}

impl LspServer {
    pub fn new() -> eyre::Result<Self> {
        // Create the transport. Includes the stdio (stdin and stdout) versions but this could
        // also be implemented to use sockets or HTTP.
        let (connection, io_threads) = Connection::stdio();

        // Run the server
        let (id, _params) = connection.initialize_start()?;

        // let init_params: InitializeParams = serde_json::from_value(params).unwrap();
        // let client_capabilities: ClientCapabilities = init_params.capabilities;
        let server_capabilities = Self::server_capabilities();

        let initialize_data = serde_json::json!({
            "capabilities": server_capabilities,
            "serverInfo": {
                "name": "dada-lsp",
                "version": "0.1"
            }
        });

        connection.initialize_finish(id, initialize_data)?;

        let db = LspServerDatabase::new(connection.sender.clone());

        Ok(Self {
            connection,
            io_threads,
            db,
        })
    }

    fn server_capabilities() -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            ..ServerCapabilities::default()
        }
    }

    pub fn main_loop(&mut self) -> eyre::Result<()> {
        for msg in &self.connection.receiver {
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    // Currently don't handle any other requests
                }
                Message::Notification(x) => {
                    if let Some(params) = as_notification::<DidOpenTextDocument>(&x) {
                        self.db.did_open(params)
                    } else if let Some(params) = as_notification::<DidChangeTextDocument>(&x) {
                        self.db.did_change(params)
                    } else if let Some(_params) = as_notification::<DidCloseTextDocument>(&x) {
                        // FIXME self.did_close(params)
                    }
                }
                Message::Response(_) => {
                    // Don't expect any of these
                }
            }
        }

        Ok(())
    }
}

fn as_notification<T>(x: &Notification) -> Option<T::Params>
where
    T: lsp_types::notification::Notification,
    T::Params: DeserializeOwned,
{
    if x.method == T::METHOD {
        let params = serde_json::from_value(x.params.clone()).unwrap_or_else(|err| {
            panic!(
                "Invalid notification\nMethod: {}\n error: {}",
                x.method, err
            )
        });
        Some(params)
    } else {
        None
    }
}

fn new_notification<T>(params: T::Params) -> Notification
where
    T: lsp_types::notification::Notification,
{
    Notification {
        method: T::METHOD.to_owned(),
        params: serde_json::to_value(&params).unwrap(),
    }
}
