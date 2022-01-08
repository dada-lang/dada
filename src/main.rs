use structopt::StructOpt;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dada_lang::Options::from_args().main().await
}
