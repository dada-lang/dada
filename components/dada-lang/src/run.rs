use std::path::PathBuf;

use dada_execute::{heap_graph::HeapGraph, machine::ProgramCounter};
use dada_ir::span::FileSpan;
use eyre::Context;
use tokio::io::AsyncWriteExt;

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
            dada_error_format::print_diagnostic(&db, &diagnostic)?;
        }

        // Find the "main" function
        match db.function_named(filename, "main") {
            Some(function) => {
                dada_execute::interpret(function, &db, &mut Kernel::new(), vec![]).await?;
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

struct Kernel {}

impl Kernel {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl dada_execute::kernel::Kernel for Kernel {
    async fn print(&mut self, _await_pc: ProgramCounter, text: &str) -> eyre::Result<()> {
        let mut stdout = tokio::io::stdout();
        let mut text = text.as_bytes();
        while !text.is_empty() {
            let written = stdout.write(text).await?;
            text = &text[written..];
        }
        return Ok(());
    }

    fn breakpoint_start(
        &mut self,
        _db: &dyn dada_execute::Db,
        _breakpoint_filename: dada_ir::filename::Filename,
        _breakpoint_index: usize,
        _generate_heap_graph: &mut dyn FnMut() -> HeapGraph,
    ) -> eyre::Result<()> {
        panic!("no breakpoints set")
    }

    fn breakpoint_end(
        &mut self,
        _db: &dyn dada_execute::Db,
        _breakpoint_filename: dada_ir::filename::Filename,
        _breakpoint_index: usize,
        _breakpoint_span: FileSpan,
        _generate_heap_graph: &mut dyn FnMut() -> HeapGraph,
    ) -> eyre::Result<()> {
        panic!("no breakpoints set")
    }
}
