use std::path::PathBuf;

use eyre::Context;

#[derive(structopt::StructOpt)]
pub struct Options {
    path: PathBuf,
}

impl Options {
    pub async fn main(&self, _crate_options: &crate::Options) -> eyre::Result<()> {
        let mut db = dada_db::Db::default();

        let contents = std::fs::read_to_string(&self.path)
            .with_context(|| format!("reading `{}`", self.path.display()))?;
        let filename = dada_ir::filename::Filename::from(&db, &self.path);
        db.update_file(filename, contents);

        for diagnostic in db.diagnostics(filename) {
            crate::format::print_diagnostic(&db, &diagnostic)?;
        }

        // Find the "main" function
        match db.function_named(filename, "main") {
            Some(function) => {
                let stdout = Box::pin(tokio::io::stdout());
                dada_execute::interpret(function, &db, stdout).await?;
            }
            None => {
                return Err(eyre::eyre!(
                    "could not find a function named `main` in `{}`",
                    self.path.display()
                ));
            }
        }

        Ok(())
    }
}
