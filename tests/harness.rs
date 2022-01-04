#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    dada_lang::Options::test_harness().main().await
}
