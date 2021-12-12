#[derive(structopt::StructOpt)]
pub struct Options {}

pub fn main(_crate_options: &crate::Options, _options: &Options) -> eyre::Result<()> {
    let mut server = dada_lsp::LspServer::new()?;
    server.main_loop()?;
    Ok(())
}
