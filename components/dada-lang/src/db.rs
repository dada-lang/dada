use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use dada_ir_ast::{
    ast::Identifier,
    inputs::{CompilationRoot, Krate, SourceFile},
};
use dada_util::{bail, Fallible, FromImpls, Map};
use salsa::{Durability, Event, Setter};

#[derive(Default)]
#[salsa::db]
pub(crate) struct Database {
    storage: salsa::Storage<Self>,
    inputs: Mutex<Inputs>,
}

#[derive(Default)]
struct Inputs {
    root: Option<CompilationRoot>,
    source_files: Map<PathBuf, SourceFile>,
    directories: Map<Krate, KrateSource>,
}

#[derive(FromImpls, Clone, Debug)]
pub enum KrateSource {
    Path(PathBuf),
}

impl Database {
    /// Access the [`CompilationRoot`], from which all crates and sources can be reached.
    pub fn root(&self) -> CompilationRoot {
        let mut inputs = self.inputs.lock().unwrap();
        if let Some(root) = inputs.root {
            return root;
        }

        // For now, just load libdada from the directory in the source tree
        let libdada = Krate::new(self, "dada".to_string());
        let libdada_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libdada");
        inputs.directories.insert(libdada, libdada_path.into());

        let root = CompilationRoot::new(self, vec![libdada]);
        inputs.root = Some(root);
        root
    }

    /// Add a crate into our list.
    pub fn add_crate(
        &mut self,
        crate_name: String,
        source: impl Into<KrateSource>,
    ) -> Fallible<()> {
        let root = self.root();
        let mut crates = root.crates(self).clone();

        if crates.iter().any(|c| *c.name(self) == crate_name) {
            bail!("crate `{}` already exists", crate_name);
        }

        let krate = Krate::new(self, crate_name);

        Mutex::get_mut(&mut self.inputs)
            .unwrap()
            .directories
            .insert(krate, source.into());

        crates.push(krate);
        root.set_crates(self)
            .with_durability(Durability::HIGH)
            .to(crates);

        Ok(())
    }

    /// Load a source-file at a given path
    pub fn source_file(&self, path: &Path) -> SourceFile {
        let mut inputs = self.inputs.lock().unwrap();

        if let Some(&opt_source_file) = inputs.source_files.get(path) {
            return opt_source_file;
        }

        let contents = match std::fs::read_to_string(path) {
            Ok(data) => Ok(data),

            Err(e) => Err(format!(
                "error reading `{}`: {}",
                path.display(),
                e.to_string()
            )),
        };

        let result = SourceFile::new(self, path.display().to_string(), contents);

        inputs.source_files.insert(path.to_path_buf(), result);

        result
    }
}

#[salsa::db]
pub trait Db: dada_check::Db {}

#[salsa::db]
impl salsa::Database for Database {
    fn salsa_event(&self, _event: &dyn Fn() -> Event) {}
}

#[salsa::db]
impl dada_check::Db for Database {
    fn root(&self) -> CompilationRoot {
        Database::root(self)
    }

    fn source_file<'db>(&'db self, krate: Krate, modules: &[Identifier<'db>]) -> SourceFile {
        let mut source = self.inputs.lock().unwrap().directories[&krate].clone();
        match &mut source {
            KrateSource::Path(path) => {
                for module in modules {
                    path.push(module.text(self));
                }
                path.set_extension("dada");
                Database::source_file(self, &*path)
            }
        }
    }
}

#[salsa::db]
impl Db for Database {}
