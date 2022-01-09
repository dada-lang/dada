use dada_error_format::format_diagnostics;
use dada_execute::kernel::BufferKernel;
use dada_ir::filename::Filename;
use wasm_bindgen::prelude::*;

/// Execute the dada code and generate output (plus compiler diagnostics).
#[wasm_bindgen]
pub async fn execute(code: String) -> String {
    let mut db = dada_db::Db::default();
    let filename = Filename::from(&db, "input.dada");
    db.update_file(filename, code);

    let diagnostics = db.diagnostics(filename);

    let output = match db.function_named(filename, "main") {
        Some(function) => {
            let kernel = BufferKernel::new();
            kernel.interpret_and_buffer(&db, function, vec![]).await;
            kernel.into_buffer()
        }
        None => {
            format!("no `main` function in `{}`", filename.as_str(&db))
        }
    };

    if !diagnostics.is_empty() {
        let diagnostics = format_diagnostics(&db, &diagnostics).unwrap();
        format!("{}\n{}", diagnostics, output)
    } else {
        output
    }
}
