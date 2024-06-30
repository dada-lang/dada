use dada_util::Fallible;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Options {}

impl Options {
    pub async fn main(self) -> Fallible<()> {
        Ok(())
    }
}
