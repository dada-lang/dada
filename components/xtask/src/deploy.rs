use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Deploy {
    #[structopt(long)]
    check: bool,
}

impl Deploy {
    pub fn main(&self) -> anyhow::Result<()> {
        let xtask_dir = cargo_path("CARGO_MANIFEST_DIR")?;
        let manifest_dir = xtask_dir.parent().unwrap().parent().unwrap();
        tracing::debug!("manifest directory: {manifest_dir:?}");
        let book_dir = manifest_dir.join("book");

        {
            let _directory = xshell::pushd(&book_dir)?;
            let npm = if cfg!(target_os = "windows") {
                "npm.cmd"
            } else {
                "npm"
            };
            xshell::Cmd::new(npm).arg("install").run()?;
            if self.check {
                xshell::Cmd::new(npm).arg("run").arg("typecheck").run()?;
                xshell::Cmd::new(npm).arg("run").arg("format:check").run()?;
            }
            xshell::Cmd::new(npm).arg("run").arg("build").run()?;
        }

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
