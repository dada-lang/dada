use dada_error_format::format_diagnostics;
use dada_execute::kernel::BufferKernel;
use dada_ir::filename::Filename;
use wasm_bindgen::prelude::*;

// Import the `window.alert` function from the Web.
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// Export a `greet` function from Rust to JavaScript, that alerts a
// hello message.
#[wasm_bindgen]
pub async fn execute(code: String) -> String {
    let mut db = dada_db::Db::default();
    let filename = Filename::from(&db, "input.dada");
    db.update_file(filename, code);

    let diagnostics = db.diagnostics(filename);
    let diagnostics = format_diagnostics(&db, &diagnostics).unwrap();

    let output = match db.function_named(filename, "main") {
        Some(function) => {
            let kernel = BufferKernel::new();
            kernel.interpret_and_buffer(&db, function).await;
            kernel.into_buffer()
        }
        None => {
            format!("no `main` function in `{}`", filename.as_str(&db))
        }
    };

    format!("{}{}", diagnostics, output)
}
