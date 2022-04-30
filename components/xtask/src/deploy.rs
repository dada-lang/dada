use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Deploy {}

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

        let mdbook_path = download_mdbook(&dada_downloads)?;
        let wasm_pack_path = download_wasm_pack(&dada_downloads)?;

        {
            let _book = xshell::pushd(&book_dir)?;
            xshell::Cmd::new(mdbook_path).arg("build").run()?;
        }

        let playground_dir = book_dir.join("book/playground");
        xshell::mkdir_p(&playground_dir)?;

        let dada_web_dir = xshell::cwd()?.join("components/dada-web");

        {
            let _directory = xshell::pushd(&dada_web_dir)?;
            xshell::Cmd::new(&wasm_pack_path)
                .arg("build")
                .arg("--target")
                .arg("web")
                .arg("--dev")
                .arg("--out-dir")
                .arg(dada_web_target_dir)
                .run()?;
        }

        //{
        //    // FIXME: run curl ourselves, write output directly into final directory?
        //    let _directory = xshell::pushd(&dada_web_dir.join("ace"))?;
        //    xshell::cmd!("make").run()?;
        //}

        copy_all_files(&dada_web_dir, "ace", &playground_dir)?;
        copy_all_files(&dada_web_dir, "pkg", &playground_dir)?;

        xshell::cp(dada_web_dir.join("index.html"), &playground_dir)?;
        xshell::cp(dada_web_dir.join("index.css"), &playground_dir)?;
        xshell::cp(dada_web_dir.join("index.js"), &playground_dir)?;
        Ok(())
    }
}

fn download_mdbook(dada_downloads: &Path) -> eyre::Result<PathBuf> {
    let version = "v0.4.15";
    let url = format!("https://github.com/rust-lang/mdBook/releases/download/{version}/mdbook-{version}-x86_64-unknown-linux-gnu.tar.gz");
    let filename = format!("mdbook-{version}.tar.gz");
    download_and_untar(dada_downloads, &url, &filename)?;
    Ok(dada_downloads.join("mdbook"))
}

fn download_wasm_pack(dada_downloads: &Path) -> eyre::Result<PathBuf> {
    let version = "v0.10.2";
    let prefix = format!("wasm-pack-{version}-x86_64-unknown-linux-musl");
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

fn copy_all_files(source_dir: &Path, subdir: &str, target_dir: &Path) -> eyre::Result<()> {
    for f in xshell::read_dir(source_dir.join(subdir))? {
        assert!(!f.is_dir()); // FIXME
        let target_subdir = &target_dir.join(subdir);
        xshell::mkdir_p(target_subdir)?;
        xshell::cp(f, target_subdir)?;
    }
    Ok(())
}
