#![feature(trait_upcasting)]
#![feature(try_blocks)]
#![allow(incomplete_features)]

use structopt::StructOpt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

mod check;
mod ide;
mod run;
mod test_harness;

const DEFAULT_LOG: &str = "warn,dada_lang=info";

#[derive(StructOpt)]
pub struct Options {
    #[structopt(long, default_value = DEFAULT_LOG)]
    log: String,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Command,
}

impl Options {
    /// Returns the options to run the default test harness.
    pub fn test_harness() -> Self {
        Options {
            log: DEFAULT_LOG.to_string(),
            cmd: Command::Test(test_harness::Options::from_args()),
        }
    }

    pub async fn main(&self) -> eyre::Result<()> {
        // Configure logging:
        let subscriber = tracing_subscriber::Registry::default()
            .with({
                // Configure which modules/level/etc using `DADA_LOG`
                // environment variable if present,
                // else the `--log` parameter.
                match std::env::var("DADA_LOG") {
                    Ok(env) => EnvFilter::new(env),
                    Err(_) => EnvFilter::new(&self.log),
                }
            })
            .with({
                // Configure the hierarchical display.
                tracing_tree::HierarchicalLayer::default()
                    .with_writer(std::io::stderr)
                    .with_indent_lines(false)
                    .with_ansi(true)
                    .with_targets(true)
                    .with_indent_amount(2)
            });
        tracing::subscriber::set_global_default(subscriber).unwrap();
        tracing_log::LogTracer::init()?;

        match &self.cmd {
            Command::Ide(command_options) => {
                ide::main(self, command_options)?;
            }
            Command::Check(command_options) => command_options.main(self)?,
            Command::Test(command_options) => command_options.main(self).await?,
            Command::Run(command_options) => command_options.main(self).await?,
        }
        Ok(())
    }
}

#[derive(StructOpt)]
pub enum Command {
    /// Pound acorns into flour for cookie dough.
    Ide(ide::Options),
    Check(check::Options),
    Test(test_harness::Options),
    Run(run::Options),
}
