use salsa::DebugWithDb;
use std::path::PathBuf;

use eyre::Context;

#[derive(structopt::StructOpt)]
pub struct Options {
    paths: Vec<PathBuf>,

    #[structopt(long)]
    print_ast: bool,
}

impl Options {
    pub fn main(&self, _crate_options: &crate::Options) -> eyre::Result<()> {
        let mut db = dada_db::Db::default();
        let mut all_diagnostics = vec![];
        for path in &self.paths {
            let contents = std::fs::read_to_string(path)
                .with_context(|| format!("reading `{}`", path.display()))?;
            let filename = dada_ir::word::Word::from(&db, path);
            db.update_file(filename, contents);
            all_diagnostics.extend(db.diagnostics(filename));

            if self.print_ast {
                for item in db.items(filename) {
                    eprintln!("{:#?}", item.debug(&db));
                }
            }
        }

        for diagnostic in all_diagnostics {
            crate::format::print_diagnostic(&db, &diagnostic)?;
        }

        Ok(())
    }
}
