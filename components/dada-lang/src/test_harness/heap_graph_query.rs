use std::path::Path;

use dada_execute::kernel::BufferKernel;
use dada_ir::input_file::InputFile;
use dada_ir::span::LineColumn;
use salsa::DebugWithDb;

use crate::test_harness::QueryKind;

use super::{Errors, Query};

impl super::Options {
    #[tracing::instrument(level = "Debug", skip(self, db, errors))]
    pub(super) async fn perform_heap_graph_query_on_db(
        &self,
        db: &mut dada_db::Db,
        path: &Path,
        query_index: usize,
        input_file: InputFile,
        query: &Query,
        errors: &mut Errors,
    ) -> eyre::Result<()> {
        assert!(matches!(query.kind, QueryKind::HeapGraph));

        assert!(input_file.breakpoint_locations(db).is_empty());
        db.set_breakpoints(input_file, vec![LineColumn::new1(query.line, query.column)]);

        let breakpoint = dada_breakpoint::breakpoint::find(
            db,
            input_file,
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

        self.check_compiled(
            db,
            &[input_file],
            |item| db.debug_bir(item),
            &path.join(format!("HeapGraph-{query_index}.bir.ref")),
        )?;

        match db.main_function(input_file) {
            Some(bir) => {
                kernel.interpret_and_buffer(db, bir, vec![]).await;
            }
            None => {
                kernel.append(&format!(
                    "no `main` function in `{}`\n",
                    input_file.name_str(db)
                ));
            }
        }

        let actual_output = kernel.take_buffer();

        let output_matched = query.message.is_match(&actual_output);

        let ref_path = path.join(format!("HeapGraph-{query_index}.ref"));
        self.check_output_against_ref_file(actual_output, &ref_path, errors)?;

        if !output_matched {
            eyre::bail!("query regex `{:?}` did not match the output", query.message);
        }

        // clear the breakpoints when we're done
        db.set_breakpoints(input_file, vec![]);

        Ok(())
    }
}
