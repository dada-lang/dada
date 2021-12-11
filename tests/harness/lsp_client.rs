use lsp_server::{Notification, Request, RequestId};
use lsp_types::notification::DidOpenTextDocument;
use lsp_types::request::Initialize;
use lsp_types::{DidOpenTextDocumentParams, TextDocumentItem, Url};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::process::{Command, Stdio};

struct ChildSession {
    child: std::process::Child,
}

impl Drop for ChildSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

impl ChildSession {
    fn spawn() -> ChildSession {
        let child = Command::new("cargo")
            .arg("run")
            .arg("--")
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
    ) -> eyre::Result<()> {
        let msg = Request {
            id,
            method: T::METHOD.to_owned(),
            params: serde_json::to_value(&params).unwrap(),
        };

        self.send_any(msg)
    }

    fn send_any(&mut self, msg: impl Serialize) -> eyre::Result<()> {
        let msg_raw = serde_json::to_string(&msg)?;

        let child_stdin = self.child.stdin.as_mut().ok_or(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "can connect to child stdin",
        ))?;

        child_stdin
            .write_all(format!("Content-Length: {}\r\n\r\n", msg_raw.len()).as_bytes())
            .expect("Failed to write to stdin");
        child_stdin
            .write_all(msg_raw.as_bytes())
            .expect("Failed to write to stdin");
        //let _ = io::stdout().flush();

        Ok(())
    }

    fn receive<T: for<'de> Deserialize<'de>>(&mut self) -> Result<T, Box<std::error::Error>> {
        let child_stdout = self.child.stdout.as_mut().ok_or(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "can connect to child stdout",
        ))?;

        let mut buffer = [0; 16];
        child_stdout.read(&mut buffer[..])?;

        let mut digits = String::new();
        let mut digit = [0; 1];
        loop {
            child_stdout.read(&mut digit[..])?;
            let char_digit = digit[0] as char;

            if char_digit.is_digit(10) {
                digits.push(char_digit);
            } else {
                let mut whitespace = [0; 3];
                child_stdout.read(&mut whitespace[..])?;
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

    fn send_init(&mut self, id: usize) -> eyre::Result<()> {
        Ok(self
            .send_request::<Initialize>(serde_json::from_str("22")?, serde_json::from_str("{}")?)?)
    }

    fn send_open(&mut self, filepath: &str) -> eyre::Result<()> {
        let contents = std::fs::read_to_string(filepath)?;
        let path = std::path::Path::new(filepath).canonicalize()?;
        Ok(
            self.send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: Url::parse(&format!(
                        "file:///{}",
                        path.to_str().ok_or(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Bad filepath"
                        ))?
                    ))?,
                    language_id: "lark".into(),
                    version: 1,
                    text: contents,
                },
            })?,
        )
    }
}

#[test]
fn find_expected_error_message() -> Result<(), Box<std::error::Error>> {
    let mut child_session = ChildSession::spawn();

    // Child that we are initialized
    child_session.send_init(100)?;

    let result = child_session.receive::<JsonRPCResponse<InitializeResult>>()?;

    assert_eq!(result.id, 100);

    // Open the document
    child_session.send_open("tests/test_files/error_type_mismatch.lark")?;

    let result = child_session.receive::<JsonRPCNotification<PublishDiagnosticsParams>>()?;

    assert_eq!(result.method, "textDocument/publishDiagnostics",);
    assert_eq!(result.params.diagnostics.len(), 1,);
    assert_eq!(
        result.params.diagnostics[0].message,
        "mismatched types (uint vs bool)",
    );

    Ok(())
}
