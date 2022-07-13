use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Deploy {
    #[structopt(long)]
    check: bool,
}

impl Deploy {
    pub fn main(&self) -> eyre::Result<()> {
        let xtask_dir = cargo_path("CARGO_MANIFEST_DIR")?;
        let manifest_dir = xtask_dir.parent().unwrap().parent().unwrap();
        tracing::debug!("manifest directory: {manifest_dir:?}");
        let book_dir = manifest_dir.join("book");
        let target_dir = manifest_dir.join("target");
        let dada_web_target_dir = target_dir.join("dada-web");
        let dada_downloads = target_dir.join("dada-downloads");
        xshell::mkdir_p(&dada_downloads)?;
        tracing::debug!("dada download directory: {dada_downloads:?}");

        let wasm_pack_path = download_wasm_pack(&dada_downloads)?;

        {
            let dada_web_dir = xshell::cwd()?.join("components/dada-web");
            let _directory = xshell::pushd(&dada_web_dir)?;
            xshell::Cmd::new(&wasm_pack_path)
                // prevent wasm-pack from invalidating cargo build cache, and viceversa
                .env("CARGO_TARGET_DIR", target_dir.join("wasm-pack"))
                .arg("build")
                .arg("--target")
                .arg("web")
                .arg("--dev")
                .arg("--out-dir")
                .arg(dada_web_target_dir)
                .run()?;
        }

        {
            let _directory = xshell::pushd(&book_dir)?;
            xshell::Cmd::new("npm").arg("install").run()?;
            if self.check {
                xshell::Cmd::new("npm").arg("run").arg("typecheck").run()?;
                xshell::Cmd::new("npm")
                    .arg("run")
                    .arg("format:check")
                    .run()?;
            }
            xshell::Cmd::new("npm").arg("run").arg("build").run()?;
        }

        Ok(())
    }
}

fn download_wasm_pack(dada_downloads: &Path) -> eyre::Result<PathBuf> {
    let arch = cfg!(target_arch);
    let vendor = cfg!(target_os);
    let version = "v0.10.2";
    let prefix = if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        format!("wasm-pack-{version}-x86_64-unknown-linux-musl")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        format!("wasm-pack-{version}-x86_64-apple-darwin")
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        format!("wasm-pack-{version}-x86_64-pc-windows-msvc")
    } else {
        eyre::bail!("no wasm-pack binary available for `{arch}` and `{vendor}`")
    };
    let filename = format!("{prefix}.tar.gz");
    let url =
        format!("https://github.com/rustwasm/wasm-pack/releases/download/{version}/{filename}");
    download_and_untar(dada_downloads, &url, &filename)?;
    Ok(dada_downloads.join(&prefix).join("wasm-pack"))
}

fn download_and_untar(dada_downloads: &Path, url: &str, file: &str) -> eyre::Result<()> {
    tracing::debug!("download_and_untar(url={url}, file={file})");
    let _pushd = xshell::pushd(dada_downloads);
    let file = Path::new(file);
    if !file.exists() {
        xshell::cmd!("curl -L -o {file} {url}").run()?;
        xshell::cmd!("tar zxf {file}").run()?;
    } else {
        tracing::debug!("file already exists");
    }
    Ok(())
}

fn cargo_path(env_var: &str) -> eyre::Result<PathBuf> {
    match std::env::var(env_var) {
        Ok(s) => {
            tracing::debug!("cargo_path({env_var}) = {s}");
            Ok(PathBuf::from(s))
        }
        Err(_) => eyre::bail!("`{}` not set", env_var),
    }
}
