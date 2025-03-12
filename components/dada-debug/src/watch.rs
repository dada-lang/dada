use std::path::{Path, PathBuf};

use notify::{RecursiveMode, Watcher};

pub struct EventStream {
    rx: tokio::sync::mpsc::UnboundedReceiver<notify::Result<notify::Event>>,
    paths: Vec<PathBuf>,
}

impl EventStream {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // Start by enqueing an event regarding the current contents
        let mut paths = vec![];
        for entry in walkdir::WalkDir::new(path) {
            let entry = entry?;
            paths.push(entry.path().to_path_buf());
        }
    
        let mut watcher = notify::recommended_watcher(WatchEvents { tx })?;
        watcher.watch(path, RecursiveMode::Recursive)?;
    
        Ok(EventStream { rx, paths })    
    }

    pub async fn next(&mut self) -> anyhow::Result<PathBuf> {
        loop {
            if let Some(p) = self.paths.pop() {
                return Ok(p);
            }

            if let Some(event) = self.rx.recv().await {
                let event = event?;
                self.paths.extend(event.paths);
                continue;
            }

            anyhow::bail!("watcher disconnected")
        }
    }
}

struct WatchEvents {
    tx: tokio::sync::mpsc::UnboundedSender<notify::Result<notify::Event>>,
}

impl notify::EventHandler for WatchEvents {
    fn handle_event(&mut self, event: notify::Result<notify::Event>) {
        self.tx.send(event).unwrap();
    }
}
