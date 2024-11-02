use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use dada_ir_ast::inputs::{CompilationRoot, CrateKind, CrateSource, SourceFile};
use dada_util::{bail, Fallible, Map};
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
}

impl Database {
    /// Access the [`CompilationRoot`], from which all crates and sources can be reached.
    pub fn root(&self) -> CompilationRoot {
        let mut inputs = self.inputs.lock().unwrap();
        if let Some(root) = inputs.root {
            return root;
        }

        // For now, just load libdada from the directory in the source tree
        let libdada_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libdada");
        let libdada =
            CrateSource::new(self, "dada".to_string(), CrateKind::Directory(libdada_path));

        let root = CompilationRoot::new(self, vec![libdada]);
        inputs.root = Some(root);
        root
    }

    pub fn add_crate(&mut self, crate_name: String, kind: CrateKind) -> Fallible<()> {
        let root = self.root();
        let mut crates = root.crates(self).clone();

        if crates.iter().any(|c| *c.name(self) == crate_name) {
            bail!("crate `{}` already exists", crate_name);
        }

        let crate_source = CrateSource::new(self, crate_name, kind);
        crates.push(crate_source);
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

    fn source_file(&self, path: &Path) -> SourceFile {
        Database::source_file(self, path)
    }
}

#[salsa::db]
impl Db for Database {}
