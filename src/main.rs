use structopt::StructOpt;

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    dada_lang::Options::from_args().main()
}
