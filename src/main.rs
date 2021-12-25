use structopt::StructOpt;

fn main() -> eyre::Result<()> {
    dada_lang::Options::from_args().main()
}
