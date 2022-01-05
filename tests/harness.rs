#[tokio::main]
async fn main() -> eyre::Result<()> {
    dada_lang::Options::test_harness().main().await
}
