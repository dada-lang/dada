use dada_ir_ast::{diagnostic::Diagnostics, inputs::SourceFile};
use dada_util::{Context, Fallible};

use crate::{db::Database, Command, CompileOptions, GlobalOptions};

pub struct Main {
    #[allow(dead_code)]
    global_options: GlobalOptions,
    db: Database,
    source_files: Vec<SourceFile>,
}

impl Main {
    pub fn new(global_options: GlobalOptions) -> Self {
        Self {
            global_options,
            db: Database::default(),
            source_files: vec![],
        }
    }

    pub async fn run(mut self, command: Command) -> Fallible<()> {
        match command {
            crate::Command::Compile { compile_options } => self.compile(&compile_options)?,
        }
        Ok(())
    }

    pub fn load_input(&mut self, input: &str) -> Fallible<SourceFile> {
        let contents = std::fs::read_to_string(input)
            .with_context(|| format!("failed to read input file `{}`", input))?;

        let source_file = SourceFile::new(&self.db, input.to_string(), contents);
        self.source_files.push(source_file);

        Ok(source_file)
    }

    pub fn compile(&mut self, compile_options: &CompileOptions) -> Fallible<()> {
        let source_file = self.load_input(&compile_options.input)?;

        let diagnostics =
            dada_ir_ast::parse::SourceFile_parse::accumulated::<Diagnostics>(&self.db, source_file);

        for diagnostic in diagnostics {
            eprintln!("{diagnostic:#?}");
        }

        Ok(())
    }
}
