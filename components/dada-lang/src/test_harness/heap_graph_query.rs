use std::path::Path;

use dada_execute::kernel::{BufferKernel, Kernel};
use dada_ir::code::Code;
use dada_ir::span::LineColumn;
use dada_ir::{code::syntax, filename::Filename};
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

        let (code, cusp_expr) = match dada_breakpoint::breakpoint::find(
            db,
            filename,
            LineColumn {
                line: query.line,
                column: query.column,
            },
        ) {
            Some(pair) => pair,
            None => eyre::bail!("could not identify the expression at this breakpoint"),
        };

        let actual_output = match db.function_named(filename, "main") {
            Some(function) => {
                let kernel = HeapGraphKernel::new(code, cusp_expr);

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
    code: Code,
    cusp_expr: syntax::Expr,
}

impl HeapGraphKernel {
    fn new(code: Code, cusp_expr: syntax::Expr) -> Self {
        Self {
            buffer: BufferKernel::new(),
            code,
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
        db: &dyn dada_execute::Db,
        stack_frame: &dada_execute::StackFrame<'_>,
        expr: syntax::Expr,
    ) -> eyre::Result<()> {
        if expr == self.cusp_expr && self.code == stack_frame.code(db) {
            let heap_graph = dada_execute::heap_graph::HeapGraph::new(db, stack_frame);
            self.buffer.append(&heap_graph.graphviz(db, true));
            Err(eyre::eyre!(BreakpointExpressionEncountered))
        } else {
            Ok(())
        }
    }
}
