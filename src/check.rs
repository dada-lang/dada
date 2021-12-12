use std::path::PathBuf;

use eyre::Context;

#[derive(structopt::StructOpt)]
pub struct Options {
    paths: Vec<PathBuf>,
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
        }

        println!("{:#?}", all_diagnostics);

        Ok(())
    }
}
