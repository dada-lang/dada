use std::path::Path;

use dada_execute::kernel::BufferKernel;
use dada_ir::filename::Filename;
use dada_ir::span::LineColumn;
use salsa::DebugWithDb;

use crate::test_harness::QueryKind;

use super::Query;

impl super::Options {
    #[tracing::instrument(level = "Debug", skip(self, in_db))]
    pub(super) async fn perform_heap_graph_query_on_db(
        &self,
        in_db: &mut dada_db::Db,
        path: &Path,
        query_index: usize,
        filename: Filename,
        query: &Query,
    ) -> eyre::Result<()> {
        assert!(matches!(query.kind, QueryKind::HeapGraph));

        // FIXME: total hack to workaround a salsa bug:
        let db = &mut dada_db::Db::default();
        db.update_file(filename, in_db.file_source(filename).clone());
        db.set_breakpoints(filename, vec![LineColumn::new1(query.line, query.column)]);

        let breakpoint = dada_breakpoint::breakpoint::find(
            db,
            filename,
            LineColumn::new1(query.line, query.column),
        );
        tracing::debug!("breakpoint={:?}", breakpoint);

        let mut kernel = BufferKernel::new()
            .breakpoint_callback(|db, kernel, record| kernel.append(&record.to_graphviz(db)));

        if let Some(breakpoint) = breakpoint {
            kernel.append(&format!(
                "# Breakpoint: {:?} at {:?}\n",
                breakpoint.expr,
                breakpoint.span(db).debug(db),
            ));
        }

        match db.function_named(filename, "main") {
            Some(function) => {
                kernel.interpret_and_buffer(db, function, vec![]).await;
            }
            None => {
                kernel.append(&format!(
                    "no `main` function in `{}`\n",
                    filename.as_str(db)
                ));
            }
        }

        let actual_output = kernel.take_buffer();

        let output_matched = query.message.is_match(&actual_output);

        let ref_path = path.join(format!("HeapGraph-{query_index}.ref"));
        self.maybe_bless_ref_file(actual_output, &ref_path)?;

        if !output_matched {
            eyre::bail!("query regex `{:?}` did not match the output", query.message);
        }

        // clear the breakpoints when we're done
        db.set_breakpoints(filename, vec![]);

        Ok(())
    }
}
