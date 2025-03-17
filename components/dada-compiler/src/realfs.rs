use std::path::{Path, PathBuf};

use crate::VirtualFileSystem;
use dada_util::{Fallible, anyhow, bail};
use url::Url;

pub struct RealFs {
    base_dir: Option<PathBuf>,
}

impl Default for RealFs {
    fn default() -> Self {
        Self::new()
    }
}

impl RealFs {
    pub fn new() -> Self {
        Self {
            base_dir: std::env::current_dir().ok(),
        }
    }

    fn validate_scheme(url: &Url) -> Fallible<PathBuf> {
        if url.scheme() != "file" {
            bail!("unsupported scheme: {}", url.scheme());
        }
        url.to_file_path()
            .map_err(|()| anyhow!("not a file path: {url}"))
    }
}

impl VirtualFileSystem for RealFs {
    fn contents(&self, url: &Url) -> Fallible<String> {
        let path = Self::validate_scheme(url)?;
        Ok(std::fs::read_to_string(&path)?)
    }

    fn exists(&self, url: &Url) -> bool {
        match Self::validate_scheme(url) {
            Ok(path) => path.exists(),
            Err(_) => false,
        }
    }

    fn path_url(&self, path: &Path) -> Fallible<Url> {
        let path = if let Some(base_dir) = &self.base_dir {
            base_dir.join(path)
        } else {
            path.to_path_buf()
        };

        Url::from_file_path(&path)
            .map_err(|()| anyhow!("unable to construct URL from `{}`", path.display()))
    }

    fn url_display(&self, url: &Url) -> String {
        match url.scheme() {
            "file" => match url.to_file_path() {
                Ok(path) => {
                    if let Some(base_dir) = &self.base_dir {
                        if let Ok(suffix) = path.strip_prefix(base_dir) {
                            return suffix.display().to_string();
                        }
                    }
                    path.display().to_string()
                }

                Err(()) => url.to_string(),
            },

            "libdada" => format!("[libdada] {}", url.path()),

            _ => url.to_string(),
        }
    }
}
