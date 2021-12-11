pub fn main() -> eyre::Result<()> {
    let mut server = dada_lsp::LspServer::new()?;
    server.main_loop()
}
