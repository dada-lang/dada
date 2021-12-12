#[derive(structopt::StructOpt)]
pub struct Options {}

pub fn main(crate_options: &crate::Options, options: &Options) -> eyre::Result<()> {
    let mut server = dada_lsp::LspServer::new()?;
    server.main_loop()?;
    Ok(())
}
