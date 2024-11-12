use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Build {}

const DADA_LSP_SERVER_CRATE: &str = "dada-lsp-server";
const WASM_TRIPLE: &str = "wasm32-wasip1-threads";
const PROFILE: &str = "debug";

impl Build {
    pub fn main(&self) -> anyhow::Result<()> {
        let xtask_dir = cargo_path("CARGO_MANIFEST_DIR")?;
        let manifest_dir = xtask_dir.parent().unwrap().parent().unwrap();
        tracing::debug!("manifest directory: {manifest_dir:?}");

        // This *should* be part of the Dockerfile, but it doesn't seem to be?
        xshell::Cmd::new("rustup")
            .arg("target")
            .arg("add")
            .arg(WASM_TRIPLE)
            .run()?;

        xshell::Cmd::new("cargo")
            .arg("build")
            .arg("-p")
            .arg(DADA_LSP_SERVER_CRATE)
            .arg("--target")
            .arg(WASM_TRIPLE)
            .run()?;

        let wasm_dir = manifest_dir.join("components/vscode/wasm");
        xshell::mkdir_p(&wasm_dir)?;

        let target_dir = manifest_dir.join("target").join(WASM_TRIPLE).join(PROFILE);
        let wasm_file = target_dir
            .join(DADA_LSP_SERVER_CRATE)
            .with_extension("wasm");

        xshell::cp(&wasm_file, &wasm_dir)?;

        Ok(())
    }
}

fn cargo_path(env_var: &str) -> anyhow::Result<PathBuf> {
    match std::env::var(env_var) {
        Ok(s) => {
            tracing::debug!("cargo_path({env_var}) = {s}");
            Ok(PathBuf::from(s))
        }
        Err(_) => anyhow::bail!("`{}` not set", env_var),
    }
}
