use std::{
    path::{Path, PathBuf}, str::FromStr, sync::{Arc, Mutex}
};

use dada_ir_ast::{
    ast::{AstFunction, AstItem, AstMember, Identifier},
    diagnostic::Diagnostic,
    inputs::{CompilationRoot, Krate, SourceFile},
};
use dada_util::{Fallible, FromImpls, Map, Set, bail, debug};
use salsa::{Database as _, Durability, Event, EventKind, Setter};
use url::Url;

mod fork;
pub use fork::Fork;
mod realfs;
pub use realfs::RealFs;
mod vfs;
pub use vfs::VirtualFileSystem;
use vfs::{ToUrl, UrlPath};

use dada_parser::prelude::*;

#[salsa::db]
pub struct Compiler {
    storage: salsa::Storage<Self>,

    /// Extra information about our inputs that the rest of Dada compiler doesn't have to know
    /// (e.g., they're URL etc).
    ///
    /// This is behind a mutex but we have an invariant that we only modify things in the mutex
    /// if we have `&mut` access to the compiler.
    inputs: Arc<Mutex<Inputs>>,

    /// Mediates all access to the file system.
    vfs: Arc<dyn VirtualFileSystem>,

    /// Directory where debug logs are written.
    debug_path: Option<PathBuf>,
}

impl Compiler {
    pub fn new(vfs: impl VirtualFileSystem, debug_path: Option<&Path>) -> Self {
        Self {
            storage: Default::default(),
            inputs: Default::default(),
            vfs: Arc::new(vfs),
            debug_path: debug_path.map(PathBuf::from),
        }
    }

    /// Create a "fork" of the compiler that has only `&self` access.
    /// This is meant to be used from another thread.
    pub fn fork(&self) -> Fork<Self> {
        Fork::from(Self {
            storage: self.storage.clone(),
            inputs: self.inputs.clone(),
            vfs: self.vfs.clone(),
            debug_path: self.debug_path.clone(),
        })
    }

    /// Load the contents of `source_url` and then open it with those contents.
    pub fn load_source_file(&mut self, source_url: &(impl ToUrl + ?Sized)) -> Fallible<SourceFile> {
        let source_url = &source_url.to_url(&*self.vfs)?;
        let contents = match self.vfs.contents(source_url) {
            Ok(s) => Ok(s),
            Err(e) => Err(e.to_string()),
        };

        self.open_source_file(source_url, contents)
    }

    /// "Open" a source file with the given contents.
    /// This will find an existing `SourceFile` if one exists and update its content.
    /// If none exists, a new `SourceFile` will be created and the containing crate will be added.
    pub fn open_source_file(
        &mut self,
        source_url: &(impl ToUrl + ?Sized),
        contents: Result<String, String>,
    ) -> Fallible<SourceFile> {
        let source_url = &source_url.to_url(&*self.vfs)?;
        let source_file = match self.get_source_file(source_url) {
            Some(v) => v,
            None => {
                self.add_crate_containing_source_file(source_url)?;
                self.get_or_create_source_file(source_url)
            }
        };

        let _ = source_file.set_contents(self).to(contents);
        Ok(source_file)
    }

    /// Get the `SourceFile` for the given path.
    /// Errors if no source file was opened yet.
    pub fn get_previously_opened_source_file(
        &mut self,
        source_url: &(impl ToUrl + ?Sized),
    ) -> Fallible<SourceFile> {
        let source_url = source_url.to_url(&*self.vfs)?;
        match self.get_source_file(&source_url) {
            Some(v) => Ok(v),
            None => {
                bail!("no source file `{source_url}`")
            }
        }
    }

    /// Given a .dada file, finds the enclosing crate and adds it into the list of crates.
    /// Given some path `a/b/c.dada`, we decide that `c` is a submodule of `a/b` if there exists
    /// a `a/b.dada`; otherwise, `c` is considered a crate of its own.
    pub fn add_crate_containing_source_file(&mut self, source_url: &Url) -> Fallible<Krate> {
        let url_path = UrlPath::from(source_url.clone());

        if !url_path.is_dada_file() {
            bail!("source URL not a `.dada` file: `{source_url}`");
        }

        // We are at `a/b/c.dada`. If there exists a path `a/b.dada`, then `c` is a submodule of `a.b`.
        // Otherwise, `c` is the root.
        let mut krate_path = url_path.clone();
        while !krate_path.is_empty() {
            krate_path = krate_path.pop();

            if !self.vfs.exists(&krate_path.dada_url()) {
                break;
            }
        }

        self.add_crate_with_root_path(&krate_path.dada_url())
    }

    /// Add a crate that is rooted in the given `dada` file.
    /// The crate is named after the file name.
    pub fn add_crate_with_root_path(&mut self, root_url: &Url) -> Fallible<Krate> {
        let url_path = UrlPath::from(root_url.clone());

        if !url_path.is_dada_file() {
            bail!("crate root path should have `.dada` extension: `{root_url}`");
        }

        let crate_name = url_path.final_module_name().to_string();

        // For a given crate, the root module would be called
        // `foo.dada` and then any submodules will be in
        // `foo/...`.
        let root_dir_path = url_path.make_directory();

        Ok(self.add_crate(crate_name.to_string(), root_dir_path.url())?)
    }

    /// Codegen the main function of a source file.
    pub fn codegen_main_fn(&self, source_file: SourceFile) -> &Option<Vec<u8>> {
        dada_codegen::codegen_main_fn(self, source_file)
    }

    /// Compute all diagnostics for a source file.
    pub fn check_all(&self, source_file: SourceFile) -> Vec<Diagnostic> {
        Self::deduplicated(check_all::accumulated::<Diagnostic>(self, source_file))
    }

    fn deduplicated(mut diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
        let mut new = Set::default();
        diagnostics.retain(|d| new.insert(d.clone()));
        diagnostics
    }

    pub fn fn_asts(&self, source_file: SourceFile) -> String {
        use std::fmt::Write;

        let mut output = String::new();

        self.attach(|_db| {
            let source = source_file.url_display(self);
            writeln!(output, "# fn parse tree from {source}").unwrap();
            writeln!(output).unwrap();

            writeln!(output, "{}", fn_asts(self, source_file)).unwrap();
        });

        output
    }

    /// Access the [`CompilationRoot`], from which all crates and sources can be reached.
    pub fn root(&self) -> CompilationRoot {
        let mut inputs = self.inputs.lock().unwrap();
        if let Some(root) = inputs.root {
            return root;
        }

        // For now, just load libdada from the directory in the source tree
        let libdada = Krate::new(self, "dada".to_string());
        inputs.directories.insert(libdada, KrateSource::Libdada);

        let root = CompilationRoot::new(self, vec![libdada]);
        inputs.root = Some(root);
        root
    }

    /// Add a crate named `crate_name` sourced at `source` into our list.
    ///
    /// We can never have two crates with the same name.
    /// If a crate `k` named `crate_name` already exists, we check if `k` has the same `source`.
    /// If so, the existing crate is returned. Otherwise, an error results.
    fn add_crate(
        &mut self,
        crate_name: String,
        new_source: impl Into<KrateSource>,
    ) -> Fallible<Krate> {
        let new_source: KrateSource = new_source.into();
        let root = self.root();
        let mut crates = root.crates(self).clone();

        if let Some(&krate) = crates.iter().find(|c| *c.name(self) == crate_name) {
            let inputs = self.inputs.lock().unwrap();
            let krate_source = inputs.directories.get(&krate).unwrap();
            if *krate_source == new_source {
                return Ok(krate);
            }
            bail!("crate `{crate_name}` already exists: {krate_source}");
        }

        let krate = Krate::new(self, crate_name);

        self.inputs
            .lock()
            .unwrap()
            .directories
            .insert(krate, new_source.into());

        crates.push(krate);
        root.set_crates(self)
            .with_durability(Durability::HIGH)
            .to(crates);

        Ok(krate)
    }

    /// If there is a source file registered at `path`, return it.
    /// Else return `None`.
    fn get_source_file(&self, url: &Url) -> Option<SourceFile> {
        self.inputs.lock().unwrap().source_files.get(url).copied()
    }

    /// Get or create a source-file at a given path.
    fn get_or_create_source_file(&self, url: &Url) -> SourceFile {
        let mut inputs = self.inputs.lock().unwrap();

        if let Some(&opt_source_file) = inputs.source_files.get(url) {
            return opt_source_file;
        }

        let contents = match self.vfs.contents(url) {
            Ok(data) => Ok(data),
            Err(e) => Err(format!("error reading `{url}`: {e}")),
        };

        let result = SourceFile::new(self, url.clone(), contents);

        inputs.source_files.insert(url.clone(), result);

        result
    }
}

#[salsa::db]
pub trait Db: dada_check::Db {}

#[salsa::db]
impl salsa::Database for Compiler {
    fn salsa_event(&self, event: &dyn Fn() -> Event) {
        if dada_util::log::is_enabled() {
            let event = event();
            match event.kind {
                EventKind::DidValidateMemoizedValue { database_key } => {
                    debug!("did_validate_memoized_value", database_key);
                }
                EventKind::WillBlockOn {
                    other_thread_id,
                    database_key,
                } => {
                    debug!("will_block_on", other_thread_id, database_key);
                }
                EventKind::WillExecute { database_key } => {
                    debug!("will_execute", database_key);
                }
                EventKind::DidSetCancellationFlag => {
                    debug!("did_set_cancellation_flag");
                }
                EventKind::WillCheckCancellation
                | EventKind::WillDiscardStaleOutput { .. }
                | EventKind::DidDiscard { .. }
                | EventKind::DidDiscardAccumulated { .. } => {}
            }
        }
    }
}

#[salsa::db]
impl dada_ir_ast::Db for Compiler {
    fn url_display(&self, url: &Url) -> String {
        self.vfs.url_display(url)
    }

    fn root(&self) -> CompilationRoot {
        Compiler::root(self)
    }

    fn source_file<'db>(&'db self, krate: Krate, modules: &[Identifier<'db>]) -> SourceFile {
        let source = self.inputs.lock().unwrap().directories[&krate].clone();
        match source {
            KrateSource::Url(url) => {
                let mut url_path = UrlPath::from(url);
                assert!(!url_path.is_dada_file());
                for module in modules {
                    url_path.push(module.text(self));
                }
                self.get_or_create_source_file(&url_path.make_dada_file().url())
            }

            KrateSource::Libdada => {
                let mut path = String::new();
                for module in modules {
                    path.push('/');
                    path.push_str(module.text(self));
                }
                path.push_str(".dada");

                if let Some(libdada_source) =
                    self.inputs.lock().unwrap().libdada_source_files.get(&path)
                {
                    return *libdada_source;
                }

                let contents = match LibDadaAsset::get(&path[1..]) {
                    Some(embedded) => match String::from_utf8(embedded.data.into_owned()) {
                        Ok(data) => Ok(data),
                        Err(e) => Err(format!("libdada file `{path}` is not utf-8: {e}")),
                    },
                    None => Err(format!("no libdada module at `{path}`")),
                };

                let libdada_url = Url::from_str(format!("libdada:///{path}").as_str()).unwrap();
                let result = SourceFile::new(self, libdada_url, contents);
                self.inputs
                    .lock()
                    .unwrap()
                    .libdada_source_files
                    .insert(path, result);

                result
            }
        }
    }

    fn debug_path(&self) -> Option<&Path> {
        self.debug_path.as_deref()
    }
}

#[salsa::db]
impl Db for Compiler {}

#[derive(rust_embed::Embed)]
#[folder = "../../libdada"]
struct LibDadaAsset;

#[derive(Default)]
struct Inputs {
    root: Option<CompilationRoot>,
    source_files: Map<Url, SourceFile>,
    libdada_source_files: Map<String, SourceFile>,
    directories: Map<Krate, KrateSource>,
}

#[derive(FromImpls, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KrateSource {
    Url(Url),

    #[no_from_impl]
    Libdada,
}

impl std::fmt::Display for KrateSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Url(url) => write!(f, "rooted at `{url}`"),
            Self::Libdada => write!(f, "built-in libdada"),
        }
    }
}

#[salsa::tracked]
fn check_all(db: &dyn Db, source_file: SourceFile) {
    use dada_check::Check;
    source_file.check(db);
}

fn fn_asts(db: &dyn Db, source_file: SourceFile) -> String {
    use std::fmt::Write;

    let mut output = String::new();

    let module = source_file.parse(db);

    for item in module.items(db) {
        match *item {
            AstItem::SourceFile(_source_file) => (),
            AstItem::Use(_use_item) => (),
            AstItem::Aggregate(class_item) => {
                writeln!(output, "## class `{}`", class_item.name(db)).unwrap();
                for member in class_item.members(db) {
                    match member {
                        AstMember::Field(_field_decl) => (),
                        AstMember::Function(function) => {
                            writeln!(output, "### fn `{}`", function.name(db).id).unwrap();
                            writeln!(output, "").unwrap();
                            writeln!(output, "{}", fn_asts_fn(db, *function)).unwrap();
                        }
                    }
                }
            }
            AstItem::Function(function) => {
                writeln!(output, "## fn `{}`", function.name(db).id).unwrap();
                writeln!(output, "").unwrap();
                writeln!(output, "{}", fn_asts_fn(db, function)).unwrap();
            }
        }
    }

    return output;

    fn fn_asts_fn<'db>(db: &'db dyn Db, function: AstFunction<'db>) -> String {
        if let Some(block) = function.body_block(db) {
            format!("{block:#?}")
        } else {
            format!("None")
        }
    }
}
