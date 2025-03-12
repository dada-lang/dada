use std::path::PathBuf;

use structopt::StructOpt;

mod view;
mod server;
mod watch;

/// Command line options for the debug server
#[derive(Debug, StructOpt)]
pub struct DebugOptions {
    #[structopt(long, default_value = "2222")]
    pub port: u32,

    #[structopt(long, default_value = "dada_debug")]
    pub serve_path: PathBuf,
}

impl DebugOptions {
    /// Create a debug server from the options
    pub fn to_server(&self) -> DebugServer {
        DebugServer {
            port: self.port,
            path: self.serve_path.clone(),
            thread: None,
        }
    }
}

/// Debug server that monitors
pub struct DebugServer {
    port: u32,
    path: PathBuf,
    thread: Option<std::thread::JoinHandle<anyhow::Result<()>>>,
}

impl DebugServer {
    /// Start the debug server if it has not already started.
    pub fn launch(mut self) -> Self {
        if self.thread.is_none() {
            let path = self.path.clone();
            self.thread = Some(std::thread::spawn(move || server::main(self.port, &path)));
        }
        self
    }

    /// Block on the debug server thread (starting one if needed).
    pub fn block_on(mut self) -> anyhow::Result<()> {
        self = self.launch();
        if let Some(thread) = self.thread {
            thread.join().unwrap()?;
        }
        Ok(())
    }
}
