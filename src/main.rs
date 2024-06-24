use dada_3p::*;

use structopt::StructOpt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dada_lang::Options::from_args().main().await
}
