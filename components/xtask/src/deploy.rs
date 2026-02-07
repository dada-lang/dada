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

        // Ensure mdbook is installed at the expected version
        ensure_mdbook_installed()?;

        // Build RFC and spec mdbooks
        {
            let rfcs_dir = manifest_dir.join("rfcs");
            let spec_dir = manifest_dir.join("spec");
            let rfcs_output_dir = book_dir.join("build").join("rfcs");
            let spec_output_dir = book_dir.join("build").join("spec");

            // Build RFCs mdbook
            {
                let _directory = xshell::pushd(&rfcs_dir)?;
                xshell::Cmd::new("mdbook")
                    .arg("build")
                    .arg("--dest-dir")
                    .arg(&rfcs_output_dir)
                    .run()?;
            }

            // Build spec mdbook
            {
                let _directory = xshell::pushd(&spec_dir)?;
                xshell::Cmd::new("mdbook")
                    .arg("build")
                    .arg("--dest-dir")
                    .arg(&spec_output_dir)
                    .run()?;
            }
        }

        // Generate rustdocs and copy to book/build/impl
        {
            let _directory = xshell::pushd(manifest_dir)?;

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

const MDBOOK_VERSION: &str = "0.5.2";

/// Ensures mdbook is installed at the expected version.
///
/// 💡 Netlify (and other CI-like environments) don't have mdbook pre-installed,
/// unlike GitHub Actions where we use peaceiris/actions-mdbook. This function
/// makes `cargo xtask deploy` self-contained by installing mdbook if needed.
fn ensure_mdbook_installed() -> anyhow::Result<()> {
    // Check if mdbook is already installed at the right version
    let output = std::process::Command::new("mdbook")
        .arg("--version")
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let version_str = String::from_utf8_lossy(&output.stdout);
            // mdbook --version outputs "mdbook v0.5.2"
            if version_str.contains(MDBOOK_VERSION) {
                tracing::info!("mdbook {MDBOOK_VERSION} already installed");
                return Ok(());
            }
            tracing::info!(
                "mdbook installed but wrong version ({}), installing {MDBOOK_VERSION}",
                version_str.trim()
            );
        }
        _ => {
            tracing::info!("mdbook not found, installing {MDBOOK_VERSION}");
        }
    }

    xshell::Cmd::new("cargo")
        .arg("install")
        .arg("mdbook")
        .arg("--version")
        .arg(MDBOOK_VERSION)
        .run()?;

    Ok(())
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
