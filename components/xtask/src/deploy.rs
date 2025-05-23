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

        // Build the Docusaurus site
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

        // Generate rustdocs and copy to book/build/impl
        {
            let _directory = xshell::pushd(&manifest_dir)?;

            // Generate rustdocs
            xshell::Cmd::new("cargo")
                .arg("doc")
                .arg("--workspace")
                .arg("--no-deps")
                .arg("--document-private-items")
                .run()?;

            // Copy rustdocs to book/build/impl
            let target_doc_dir = manifest_dir.join("target").join("doc");
            let book_impl_dir = book_dir.join("build").join("impl");

            // Remove existing impl directory if it exists
            if book_impl_dir.exists() {
                std::fs::remove_dir_all(&book_impl_dir)?;
            }

            // Copy the entire doc directory
            copy_dir_recursive(&target_doc_dir, &book_impl_dir)?;
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

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
