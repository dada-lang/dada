use std::path::PathBuf;

use eyre::Context;
use salsa::DebugWithDb;

#[derive(structopt::StructOpt)]
pub struct Options {
    paths: Vec<PathBuf>,

    #[structopt(long)]
    log_syntax_tree: bool,

    #[structopt(long)]
    log_validated_tree: bool,

    #[structopt(long)]
    log_bir: bool,
}

impl Options {
    pub fn main(&self, _crate_options: &crate::Options) -> eyre::Result<()> {
        let mut db = dada_db::Db::default();
        let mut all_diagnostics = vec![];
        for path in &self.paths {
            let contents = std::fs::read_to_string(path)
                .with_context(|| format!("reading `{}`", path.display()))?;
            let filename = dada_ir::filename::Filename::from(&db, path);
            db.update_file(filename, contents);
            all_diagnostics.extend(db.diagnostics(filename));

            if self.log_syntax_tree {
                for item in db.items(filename) {
                    if let Some(tree) = db.debug_syntax_tree(item) {
                        tracing::info!("syntax tree for {:?} is {:#?}", item.debug(&db), tree);
                    }
                }
            }

            if self.log_validated_tree {
                for item in db.items(filename) {
                    if let Some(tree) = db.debug_validated_tree(item) {
                        tracing::info!("validated tree for {:?} is {:#?}", item.debug(&db), tree);
                    }
                }
            }

            if self.log_bir {
                for item in db.items(filename) {
                    if let Some(tree) = db.debug_bir(item) {
                        tracing::info!("BIR for {:?} is {:#?}", item.debug(&db), tree);
                    }
                }
            }
        }

        for diagnostic in all_diagnostics {
            dada_error_format::print_diagnostic(&db, &diagnostic)?;
        }

        Ok(())
    }
}
