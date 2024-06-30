use dada_util::Fallible;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Fallible<()> {
    dada_lang::Options::from_args().main().await
}
