use std::sync::mpsc::Sender;

use dada_ir_ast::DebugEvent;
use structopt::StructOpt;

mod hbs;
mod view;
mod server;
mod error;
mod root;

/// Command line options for the debug server
#[derive(Debug, StructOpt)]
pub struct DebugOptions {
    #[structopt(long, default_value = "2222")]
    pub port: u32,
}

impl DebugOptions {
    /// Create a debug server from the options
    pub fn to_server(&self) -> DebugServer {
        DebugServer {
            port: self.port,
            thread: None,
        }
    }
}

/// Debug server that monitors
pub struct DebugServer {
    port: u32,
    thread: Option<std::thread::JoinHandle<anyhow::Result<()>>>,
}

impl DebugServer {
    /// Start the debug server, panicking if already launched.
    /// 
    /// Returns a port where you should send debug events.
    pub fn launch(&mut self) -> Sender<DebugEvent> {
        assert!(self.thread.is_none());
        let (debug_tx, debug_rx) = std::sync::mpsc::channel();
        let port = self.port;
        self.thread = Some(std::thread::spawn(move || server::main(port, debug_rx)));
        debug_tx
    }

    /// Block on the debug server thread (if it has been launched)
    pub fn block_on(self) -> anyhow::Result<()> {
        if let Some(thread) = self.thread {
            thread.join().unwrap()?;
        }
        Ok(())
    }
}
