#![feature(trait_upcasting)]
#![feature(panic_payload_as_str)]

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

    Test {
        #[structopt(flatten)]
        test_options: TestOptions,
    },
}

#[derive(Debug, StructOpt)]
pub struct CompileOptions {
    /// Main source file to compile.
    input: String,
}

#[derive(Debug, StructOpt)]
pub struct TestOptions {
    /// Test file(s) or directory
    inputs: Vec<String>,
}

impl Options {
    pub fn main(self) -> Fallible<()> {
        main_lib::Main::new(self.global_options).run(self.command)
    }
}
