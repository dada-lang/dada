use std::path::PathBuf;

use dada_execute::{heap_graph::HeapGraph, machine::ProgramCounter};
use dada_ir::span::FileSpan;
use eyre::Context;
use regex::Regex;
use salsa::DebugWithDb;
use tokio::io::AsyncWriteExt;

#[derive(structopt::StructOpt)]
pub struct Options {
    /// Path to `.dada` file to execute
    path: PathBuf,

    /// Instead of executing, print BIR for items whose names match the given regex
    #[structopt(long)]
    bir: Option<Regex>,

    /// Instead of executing, print validated tree for items whose names match the given regex
    #[structopt(long)]
    validated: Option<Regex>,
}

impl Options {
    pub async fn main(&self, _crate_options: &crate::Options) -> eyre::Result<()> {
        let mut db = dada_db::Db::default();

        let contents = std::fs::read_to_string(&self.path)
            .with_context(|| format!("reading `{}`", self.path.display()))?;
        let input_file = db.new_input_file(&self.path, contents);

        for diagnostic in db.diagnostics(input_file) {
            dada_error_format::print_diagnostic(&db, &diagnostic)?;
        }

        let mut should_execute = true;

        if let Some(name_regex) = &self.validated {
            for item in db.items(input_file) {
                let name = item.name(&db).as_str(&db);
                if name_regex.is_match(name) {
                    if let Some(tree) = db.debug_validated_tree(item) {
                        tracing::info!("Validated tree for {:?} is {:#?}", item.debug(&db), tree);
                    }
                }
            }
            should_execute = false;
        }

        if let Some(name_regex) = &self.bir {
            for item in db.items(input_file) {
                let name = item.name(&db).as_str(&db);
                if name_regex.is_match(name) {
                    if let Some(tree) = db.debug_bir(item) {
                        tracing::info!("BIR for {:?} is {:#?}", item.debug(&db), tree);
                    }
                }
            }
            should_execute = false;
        }

        // Find the "main" function
        if should_execute {
            match db.main_function(input_file) {
                Some(bir) => {
                    dada_execute::interpret(bir, &db, &mut Kernel::new(), vec![]).await?;
                }
                None => {
                    return Err(eyre::eyre!(
                        "could not find a function named `main` in `{}`",
                        self.path.display()
                    ));
                }
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
        _breakpoint_input_file: dada_ir::input_file::InputFile,
        _breakpoint_index: usize,
        _generate_heap_graph: &mut dyn FnMut() -> HeapGraph,
    ) -> eyre::Result<()> {
        panic!("no breakpoints set")
    }

    fn breakpoint_end(
        &mut self,
        _db: &dyn dada_execute::Db,
        _breakpoint_input_file: dada_ir::input_file::InputFile,
        _breakpoint_index: usize,
        _breakpoint_span: FileSpan,
        _generate_heap_graph: &mut dyn FnMut() -> HeapGraph,
    ) -> eyre::Result<()> {
        panic!("no breakpoints set")
    }
}
