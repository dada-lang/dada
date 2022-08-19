use lsp_server::{Notification, Request, RequestId};
use lsp_types::notification::{DidOpenTextDocument, PublishDiagnostics};
use lsp_types::request::Initialize;
use lsp_types::{ClientCapabilities, Diagnostic, DidOpenTextDocumentParams, TextDocumentItem, Url};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
pub(crate) struct ChildSession {
    child: std::process::Child,
}

impl Drop for ChildSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

impl ChildSession {
    pub fn spawn() -> ChildSession {
        let cur_exe = std::env::current_exe().expect("Failed to get current executable path");
        let child = Command::new(cur_exe)
            .arg("ide")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn child process");

        ChildSession { child }
    }
    /// Helper function to do the work of sending a result back to the IDE
    fn send_notification<T: lsp_types::notification::Notification>(
        &mut self,
        params: T::Params,
    ) -> eyre::Result<()> {
        let msg = Notification {
            method: T::METHOD.to_owned(),
            params: serde_json::to_value(&params).unwrap(),
        };

        self.send_any(msg)
    }

    fn send_request<T: lsp_types::request::Request>(
        &mut self,
        id: RequestId,
        params: T::Params,
    ) -> eyre::Result<T::Result> {
        let msg = Request {
            id: id.clone(),
            method: T::METHOD.to_owned(),
            params: serde_json::to_value(&params).unwrap(),
        };

        self.send_any(msg)?;

        let response: JsonRpcResponse<T::Result> = self.receive()?;

        assert_eq!(response.id, id);
        Ok(response.result)
    }

    fn send_any(&mut self, msg: impl Serialize) -> eyre::Result<()> {
        let msg_raw = serde_json::to_string(&msg)?;

        let child_stdin = self.child.stdin.as_mut().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "can connect to child stdin")
        })?;

        child_stdin
            .write_all(format!("Content-Length: {}\r\n\r\n", msg_raw.len()).as_bytes())
            .expect("Failed to write to stdin");
        child_stdin
            .write_all(msg_raw.as_bytes())
            .expect("Failed to write to stdin");
        //let _ = io::stdout().flush();

        Ok(())
    }

    fn receive_notification<T: lsp_types::notification::Notification>(
        &mut self,
    ) -> eyre::Result<T::Params> {
        let msg: Notification = self.receive()?;
        assert_eq!(msg.method, T::METHOD);
        Ok(serde_json::from_value(msg.params)?)
    }

    fn receive<T: for<'de> Deserialize<'de>>(&mut self) -> eyre::Result<T> {
        let child_stdout = self.child.stdout.as_mut().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "can connect to child stdout",
            )
        })?;

        let mut buffer = [0; 16];
        child_stdout.read_exact(&mut buffer[..])?;

        let mut digits = String::new();
        let mut digit = [0; 1];
        loop {
            child_stdout.read_exact(&mut digit[..])?;
            let char_digit = digit[0] as char;

            if char_digit.is_ascii_digit() {
                digits.push(char_digit);
            } else {
                let mut whitespace = [0; 3];
                child_stdout.read_exact(&mut whitespace[..])?;
                break;
            }
        }
        let num_bytes: usize = digits.trim().parse()?;
        let mut buffer = vec![0u8; num_bytes];
        let _ = child_stdout.read_exact(&mut buffer);

        let buffer_string = String::from_utf8(buffer)?;

        let response: T = serde_json::from_str(&buffer_string)?;
        Ok(response)
    }

    #[allow(deprecated)]
    pub fn send_init(&mut self) -> eyre::Result<()> {
        self.send_request::<Initialize>(
            serde_json::from_str("22")?,
            lsp_types::InitializeParams {
                process_id: None,
                root_path: None,
                root_uri: None,
                initialization_options: None,
                capabilities: ClientCapabilities {
                    workspace: None,
                    text_document: None,
                    window: None,
                    experimental: None,
                    general: None,
                },
                trace: None,
                workspace_folders: None,
                client_info: None,
                locale: None,
            },
        )?;

        self.send_notification::<lsp_types::notification::Initialized>(
            lsp_types::InitializedParams {},
        )?;

        Ok(())
    }

    pub fn send_open(&mut self, filepath: &Path) -> eyre::Result<()> {
        let contents = std::fs::read_to_string(filepath)?;
        let path = std::path::Path::new(filepath).canonicalize()?;

        self.send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: Url::parse(&format!(
                    "file:///{}",
                    path.to_str().ok_or_else(|| std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Bad filepath"
                    ))?
                ))?,
                language_id: "dada".into(),
                version: 1,
                text: contents,
            },
        })
    }

    pub fn receive_errors(&mut self) -> eyre::Result<Vec<Diagnostic>> {
        let result = self.receive_notification::<PublishDiagnostics>()?;
        Ok(result.diagnostics)
    }
}

/// The command given by the IDE to the LSP server. These represent the actions of the user in the IDE,
/// as well as actions the IDE might perform as a result of user actions (like cancelling a task)
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[allow(non_camel_case_types)]
pub enum LspCommand {
    initialize {
        id: usize,
        params: Box<lsp_types::InitializeParams>,
    },
    initialized,
    #[serde(rename = "textDocument/didOpen")]
    didOpen {
        params: lsp_types::DidOpenTextDocumentParams,
    },
    #[serde(rename = "textDocument/didChange")]
    didChange {
        params: lsp_types::DidChangeTextDocumentParams,
    },
    #[serde(rename = "textDocument/hover")]
    hover {
        id: usize,
        params: lsp_types::TextDocumentPositionParams,
    },
    #[serde(rename = "textDocument/completion")]
    completion {
        id: usize,
        params: lsp_types::CompletionParams,
    },
    #[serde(rename = "textDocument/definition")]
    definition {
        id: usize,
        params: lsp_types::TextDocumentPositionParams,
    },
    #[serde(rename = "textDocument/references")]
    references {
        id: usize,
        params: lsp_types::TextDocumentPositionParams,
    },
    #[serde(rename = "textDocument/rename")]
    rename {
        id: usize,
        params: lsp_types::RenameParams,
    },
    #[serde(rename = "$/cancelRequest")]
    cancelRequest {
        params: lsp_types::CancelParams,
    },
    #[serde(rename = "completionItem/resolve")]
    completionItemResolve {
        id: usize,
        // box to address clippy::large_enum_variant
        params: Box<lsp_types::CompletionItem>,
    },
}

/// A wrapper for responses back to the IDE from the LSP service. These must follow
/// the JSON 2.0 RPC spec
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    jsonrpc: String,
    pub id: RequestId,
    pub result: T,
}
