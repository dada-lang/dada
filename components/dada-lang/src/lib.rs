#![feature(panic_payload_as_str)]

use dada_debug::DebugOptions;
use dada_ir_ast::diagnostic::RenderOptions;
use dada_util::Fallible;
use structopt::StructOpt;

mod main_lib;

use dada_compiler::Db;

#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(flatten)]
    global_options: GlobalOptions,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
pub struct GlobalOptions {
    #[structopt(long)]
    no_color: bool,
}

impl GlobalOptions {
    pub(crate) fn test_options() -> Self {
        Self { no_color: false }
    }

    pub(crate) fn render_opts(&self) -> RenderOptions {
        RenderOptions {
            no_color: self.no_color,
        }
    }
}

#[derive(Debug, StructOpt)]
pub enum Command {
    Compile {
        #[structopt(flatten)]
        compile_options: CompileOptions,
    },

    Run {
        #[structopt(flatten)]
        run_options: RunOptions,
    },

    Test {
        #[structopt(flatten)]
        test_options: TestOptions,
    },

    Debug {
        #[structopt(flatten)]
        debug_options: DebugOptions,

        #[structopt(flatten)]
        compile_options: CompileOptions,
    },
}

#[derive(Debug, StructOpt)]
pub struct CompileOptions {
    /// Main source file to compile.
    input: String,
    #[structopt(long)]
    emit_wasm: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct RunOptions {
    #[structopt(flatten)]
    compile_options: CompileOptions,
}

#[derive(Debug, StructOpt)]
pub struct TestOptions {
    /// Print each test as we run it
    #[structopt(long, short)]
    verbose: bool,

    /// Test file(s) or directory
    inputs: Vec<String>,
}

impl Options {
    pub fn main(self) -> Fallible<()> {
        main_lib::Main::new(self.global_options).run(self.command)
    }
}
