use std::path::{Path, PathBuf};

use crate::VirtualFileSystem;
use dada_util::{anyhow, bail, Fallible};
use url::Url;

pub struct RealFs;

impl RealFs {
    pub fn url(path: &Path) -> Fallible<Url> {
        Url::from_file_path(path)
            .map_err(|()| anyhow!("unable to construct URL from `{}`", path.display()))
    }

    fn validate_scheme(url: &Url) -> Fallible<PathBuf> {
        if url.scheme() != "file" {
            bail!("unsupported scheme: {}", url.scheme());
        }
        Ok(url
            .to_file_path()
            .map_err(|()| anyhow!("not a file path: {url}"))?)
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
}
