//! The command line REPL driver.

#![allow(unused)]
#![allow(clippy::all)]

use dada_execute::heap_graph::HeapGraph;
use dada_execute::machine::ProgramCounter;
use dada_ir::{filename::Filename, span::FileSpan};
use dada_repl::eval::Evaluator;
use dada_repl::loader;
use dada_repl::read::{Command, Reader, Step};
use tokio::io::AsyncWriteExt;

#[derive(structopt::StructOpt)]
pub struct Options {}

impl Options {
    pub async fn main(&self, _crate_options: &crate::Options) -> eyre::Result<()> {
        let mut rl = rustyline::Editor::<()>::new();
        let mut stderr = tokio::io::stderr();

        'reset: loop {
            let mut db = dada_db::Db::default();
            let mut kernel = Kernel::new();
            let mut reader = Reader::new();
            let mut evaluator = Evaluator::new(&mut db, &mut kernel);

            'nextline: loop {
                let (rl_, line) = match reader.doing_multiline() {
                    false => read_line(rl, ">>> ").await?,
                    true => read_line(rl, "... ").await?,
                };
                rl = rl_;

                let line = match line {
                    Ok(line) => {
                        rl.add_history_entry(&line);
                        line
                    }
                    Err(rustyline::error::ReadlineError::Eof) => {
                        break 'reset;
                    }
                    Err(rustyline::error::ReadlineError::Interrupted) => {
                        stderr.write_all(b"interrupted\n").await?;
                        reader.interrupt();
                        continue 'nextline;
                    }
                    Err(e) => {
                        stderr
                            .write_all(format!("error: {}\n", e).as_bytes())
                            .await?;
                        continue 'nextline;
                    }
                };

                let next = reader.step(line);

                let mut suggestion = None;
                let eval_res = match next {
                    Err(e) => {
                        stderr
                            .write_all(format!("error: {}\n", e).as_bytes())
                            .await?;
                        Ok(())
                    }
                    Ok(Step::ReadMore) => {
                        Ok(())
                    }
                    Ok(Step::EvalExpr(text)) => {
                        let res = evaluator.eval_expr(text).await;
                        match res {
                            Ok(None) => Ok(()),
                            Ok(Some(s)) => {
                                suggestion = Some(s.suggestion);
                                Err(s.error)
                            }
                            Err(e) => Err(e),
                        }
                    }
                    Ok(Step::EvalBindingExpr { name, text }) => {
                        evaluator.eval_binding_expr(name, text).await
                    }
                    Ok(Step::AddItem { name, text }) => evaluator.add_item(name, text),
                    Ok(Step::ExecCommand(Command::Exit)) => {
                        break 'reset;
                    }
                    Ok(Step::ExecCommand(Command::Reset)) => {
                        continue 'reset;
                    }
                    Ok(Step::ExecCommand(Command::Help)) => {
                        let help_info = dada_repl::help::HelpInfo::new();
                        for (cmd, desc) in help_info.commands {
                            let line = format!("{cmd} - {desc}\n");
                            stderr.write_all(line.as_bytes()).await?;
                        }
                        Ok(())
                    }
                    Ok(Step::ExecCommand(Command::Load(path))) => {
                        try {
                            let source = tokio::fs::read_to_string(&path).await?;
                            loader::load(&mut reader, &mut evaluator, &source).await?;
                        }
                    }
                    Ok(Step::ExecCommand(Command::DumpSource)) => {
                        let source = evaluator.get_source();
                        stderr.write_all(format!("{}\n", source).as_bytes()).await?;
                        Ok(())
                    }
                };

                if let Err(e) = eval_res {
                    stderr.write_all(format!("{}\n", e).as_bytes()).await?;
                }

                if let Some(suggestion) = suggestion {
                    stderr.write_all(format!("Suggestion: {suggestion}\n").as_bytes()).await?;
                }
            }
        }

        Ok(())
    }
}

async fn read_line(
    mut rl: rustyline::Editor<()>,
    prompt: &'static str,
) -> eyre::Result<(rustyline::Editor<()>, rustyline::Result<String>)> {
    Ok(tokio::task::spawn_blocking(move || {
        let line = rl.readline(prompt);
        (rl, line)
    })
    .await?)
}

pub struct Kernel {}

impl Kernel {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl dada_execute::kernel::Kernel for Kernel {
    async fn print(&mut self, await_pc: ProgramCounter, text: &str) -> eyre::Result<()> {
        let mut stdout = tokio::io::stdout();
        let mut text = text.as_bytes();
        while !text.is_empty() {
            let written = stdout.write(text).await?;
            text = &text[written..];
        }
        return Ok(());
    }

    fn breakpoint_start(
        &mut self,
        db: &dyn dada_execute::Db,
        breakpoint_filename: Filename,
        breakpoint_index: usize,
        generate_heap_graph: &mut dyn FnMut() -> HeapGraph,
    ) -> eyre::Result<()> {
        Ok(())
    }

    fn breakpoint_end(
        &mut self,
        db: &dyn dada_execute::Db,
        breakpoint_filename: Filename,
        breakpoint_index: usize,
        breakpoint_span: FileSpan,
        generate_heap_graph: &mut dyn FnMut() -> HeapGraph,
    ) -> eyre::Result<()> {
        Ok(())
    }
}
