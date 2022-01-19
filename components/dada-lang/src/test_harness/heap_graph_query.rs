use std::path::Path;

use dada_execute::kernel::{BufferKernel, Kernel};
use dada_ir::span::LineColumn;
use dada_ir::{code::syntax, filename::Filename};
use dada_parse::prelude::DadaParseItemExt;
use eyre::Context;
use thiserror::Error;

use crate::test_harness::QueryKind;

use super::{Errors, Query};

impl super::Options {
    pub(super) async fn perform_heap_graph_query_on_db(
        &self,
        db: &dada_db::Db,
        path: &Path,
        query_index: usize,
        filename: Filename,
        query: &Query,
        errors: &mut Errors,
    ) -> eyre::Result<()> {
        assert!(matches!(query.kind, QueryKind::HeapGraph));

        let offset = dada_ir::lines::offset(
            db,
            filename,
            LineColumn {
                line: query.line,
                column: query.column,
            },
        );

        let item = match dada_breakpoint::breakpoint::find_item(db, filename, offset) {
            Some(item) => item,
            None => eyre::bail!("query point is not within any item"),
        };

        let syntax_tree = match item.syntax_tree(db) {
            Some(syntax_tree) => syntax_tree,
            None => eyre::bail!(
                "item `{}` has no associated syntax tree",
                item.name(db).as_str(db)
            ),
        };

        let cusp_expr = match dada_breakpoint::breakpoint::find_syntax_expr(db, syntax_tree, offset)
        {
            Some(cusp_expr) => cusp_expr,
            None => eyre::bail!("could not identify the expression at this breakpoint"),
        };

        let actual_output = match db.function_named(filename, "main") {
            Some(function) => {
                let kernel = HeapGraphKernel::new(cusp_expr);

                // Execute the function and check that we encounter the breakpoint.
                // If we did, then the heap-graph should be contained in the output.
                match dada_execute::interpret(function, db, &kernel, vec![]).await {
                    Ok(()) => eyre::bail!("never encountered the breakpoint"),
                    Err(e) => match e.downcast() {
                        Ok(BreakpointExpressionEncountered) => kernel.buffer.into_buffer(),
                        Err(e) => Err(e).with_context(|| "encountered error but not breakpoint")?,
                    },
                }
            }
            None => {
                format!("no `main` function in `{}`", filename.as_str(db))
            }
        };

        let output_matched = query.message.is_match(&actual_output);

        let ref_path = path.join(format!("HeapGraph-{query_index}.ref"));
        self.check_output_against_ref_file(actual_output, &ref_path, errors)?;

        if !output_matched {
            eyre::bail!("query regex `{:?}` did not match the output", query.message);
        }

        Ok(())
    }
}

struct HeapGraphKernel {
    buffer: BufferKernel,
    cusp_expr: syntax::Expr,
}

impl HeapGraphKernel {
    fn new(cusp_expr: syntax::Expr) -> Self {
        Self {
            buffer: BufferKernel::new(),
            cusp_expr,
        }
    }
}

#[derive(Error, Debug)]
#[error("breakpoint expression encountered")]
struct BreakpointExpressionEncountered;

#[async_trait::async_trait]
impl Kernel for HeapGraphKernel {
    async fn print(&self, text: &str) -> eyre::Result<()> {
        self.buffer.print(text).await
    }

    fn on_cusp(
        &self,
        _db: &dyn dada_execute::Db,
        _stack_frame: &dada_execute::StackFrame<'_>,
        expr: syntax::Expr,
    ) -> eyre::Result<()> {
        if expr == self.cusp_expr {
            Err(eyre::eyre!(BreakpointExpressionEncountered))
        } else {
            Ok(())
        }
    }
}
