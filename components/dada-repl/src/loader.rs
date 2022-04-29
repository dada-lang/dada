//! This is a hack to load dada code into the repl.

use crate::eval::Evaluator;
use crate::read::{Command, Reader, Step};

pub async fn load(
    reader: &mut Reader,
    evaluator: &mut Evaluator<'_>,
    source: &str,
) -> eyre::Result<()> {
    for line in source.lines() {
        let next = reader.step(line.into())?;

        match next {
            Step::ReadMore => {
                continue;
            }
            Step::EvalExpr(text) => {
                evaluator.eval_expr(text).await?;
            }
            Step::EvalBindingExpr { name, text } => evaluator.eval_binding_expr(name, text).await?,
            Step::AddItem { name, text } => evaluator.add_item(name, text)?,
            Step::ExecCommand(_) => {
                return Err(eyre::eyre!("repl commands not allowed in loaded source"));
            }
        }
    }

    Ok(())
}
