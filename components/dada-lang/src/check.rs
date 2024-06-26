use std::path::PathBuf;

use eyre::Context;
use salsa::DebugWithDb;

#[derive(structopt::StructOpt)]
pub struct Options {
    /// Paths to `.dada` files to check
    paths: Vec<PathBuf>,

    /// Log the syntax tree
    #[structopt(long)]
    log_syntax_tree: bool,

    /// Log the validated tree
    #[structopt(long)]
    log_validated_tree: bool,

    /// Log the BIR
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
            let input_file = db.new_input_file(path, contents);
            all_diagnostics.extend(db.diagnostics(input_file));

            if self.log_syntax_tree {
                for item in db.items(input_file) {
                    if let Some(tree) = db.debug_syntax_tree(item) {
                        tracing::info!("syntax tree for {:?} is {:#?}", item.debug(&db), tree);
                    }
                }
            }

            if self.log_validated_tree {
                for item in db.items(input_file) {
                    if let Some(tree) = db.debug_validated_tree(item) {
                        tracing::info!("validated tree for {:?} is {:#?}", item.debug(&db), tree);
                    }
                }
            }

            if self.log_bir {
                for item in db.items(input_file) {
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
