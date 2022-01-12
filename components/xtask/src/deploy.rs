use std::path::Path;

use mdbook::MDBook;
use structopt::StructOpt;
use wasm_pack;

#[derive(StructOpt)]
pub struct Deploy {}

impl Deploy {
    pub fn main(&self) -> eyre::Result<()> {
        let book_dir = xshell::cwd()?.join("book");

        {
            let _book = xshell::pushd(&book_dir)?;
            // FIXME: I can't use `?` here because how to convert between anyhow/eyre
            let md = MDBook::load(&book_dir).expect("mdbook failed to load");
            md.build().expect("mdbook failed to build");
        }

        let playground_dir = book_dir.join("book/playground");
        xshell::mkdir_p(&playground_dir)?;

        let dada_web_dir = xshell::cwd()?.join("components/dada-web");

        {
            let _directory = xshell::pushd(&dada_web_dir)?;
            let args =
                wasm_pack::Cli::from_iter(vec!["wasm-pack", "build", "--target", "web", "--dev"]);
            wasm_pack::command::run_wasm_pack(args.cmd).expect("wasm-pack failed");
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

fn copy_all_files(source_dir: &Path, subdir: &str, target_dir: &Path) -> eyre::Result<()> {
    for f in xshell::read_dir(source_dir.join(subdir))? {
        assert!(!f.is_dir()); // FIXME
        let target_subdir = &target_dir.join(subdir);
        xshell::mkdir_p(target_subdir)?;
        xshell::cp(f, target_subdir)?;
    }
    Ok(())
}
