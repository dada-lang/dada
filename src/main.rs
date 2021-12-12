use structopt::StructOpt;

fn main() -> eyre::Result<()> {
    dada::Options::from_args().main()
}
