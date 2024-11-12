use structopt::StructOpt;
use tracing_subscriber::{prelude::*, EnvFilter};

mod build;

fn main() -> anyhow::Result<()> {
    Options::from_args().main()
}

#[derive(StructOpt)]
pub struct Options {
    #[structopt(long, default_value = "info")]
    log: String,

    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    command: Command,
}

#[derive(StructOpt)]
pub enum Command {
    Build {
        #[structopt(flatten)]
        options: build::Build,
    },
}

impl Options {
    fn main(&self) -> anyhow::Result<()> {
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

        match &self.command {
            Command::Build { options } => options.main(),
        }
    }
}
