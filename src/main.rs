use dada_util::Fallible;
use structopt::StructOpt;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Fallible<()> {
    dada_lang::Options::from_args().main()
}
