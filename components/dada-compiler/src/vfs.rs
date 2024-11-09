use std::path::Path;

use dada_util::Fallible;
use url::Url;

pub trait VirtualFileSystem: Send + Sync + 'static {
    /// Loads the contents of the given URL (or fail with a useful error).
    fn contents(&self, url: &Url) -> Fallible<String>;

    /// True if the given URL exists.
    fn exists(&self, url: &Url) -> bool;

    /// (Try to) convert a path on the local file system to a URL
    fn path_url(&self, path: &Path) -> Fallible<Url>;

    /// Return a string for the way we should display `url` to the user
    fn url_display(&self, url: &Url) -> String;
}

#[derive(Clone, Debug)]
pub(crate) struct UrlPath {
    source_url: Url,
    paths: Vec<String>,
}

impl From<Url> for UrlPath {
    fn from(url: Url) -> Self {
        let paths = url
            .path()
            .split("/")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        Self {
            source_url: url,
            paths,
        }
    }
}

impl UrlPath {
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    /// Removes the final component (if any).
    /// Result will never be a dada file.
    pub fn pop(mut self) -> Self {
        self.paths.pop();
        self
    }

    /// Append a component.
    pub fn push(&mut self, s: &str) {
        assert!(!self.is_dada_file());
        self.paths.push(s.to_string());
    }

    /// True if final component ends in `.dada`
    pub fn is_dada_file(&self) -> bool {
        let Some(last) = self.paths.last() else {
            return false;
        };

        last.ends_with(".dada")
    }

    /// True if final component ends in `.dada`
    pub fn final_module_name(&self) -> &str {
        assert!(self.is_dada_file());

        let Some(last) = self.paths.last() else {
            unreachable!()
        };

        &last[0..last.len() - ".dada".len()]
    }
    /// Remove `.dada` suffix from final component.
    ///
    /// # Panics
    ///
    /// Panics if final component does not end in `.dada`
    pub fn make_directory(mut self) -> Self {
        assert!(self.is_dada_file());

        let Some(last) = self.paths.last_mut() else {
            unreachable!()
        };

        assert!(last.ends_with(".dada"));
        last.truncate(last.len() - ".dada".len());
        self
    }

    /// Add `.dada` suffix to final component.
    ///
    /// # Panics
    ///
    /// Panics if final component already has `.dada`
    pub fn make_dada_file(mut self) -> Self {
        assert!(!self.is_dada_file());

        let Some(last) = self.paths.last_mut() else {
            self.paths.push(".dada".to_string());
            return self;
        };

        last.push_str(".dada");
        self
    }

    /// Create a URL for this path with a `.dada` extension
    ///
    /// # Panics
    ///
    /// Panics if this path already has a `.dada` extension
    pub fn dada_url(&self) -> Url {
        assert!(!self.is_dada_file());
        let mut path = self.paths.join("/");
        path.push_str(".dada");

        let mut url = self.source_url.clone();
        url.set_path(&path);

        url
    }

    /// Convert this path back into a URL
    pub fn url(&self) -> Url {
        let path = self.paths.join("/");
        let mut url = self.source_url.clone();
        url.set_path(&path);
        url
    }
}

pub trait ToUrl {
    fn to_url(&self, vfs: &dyn VirtualFileSystem) -> Fallible<Url>;
}

impl ToUrl for Url {
    fn to_url(&self, _vfs: &dyn VirtualFileSystem) -> Fallible<Url> {
        Ok(self.clone())
    }
}

impl ToUrl for Path {
    fn to_url(&self, vfs: &dyn VirtualFileSystem) -> Fallible<Url> {
        vfs.path_url(self)
    }
}
