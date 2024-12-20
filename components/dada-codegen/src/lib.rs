#![feature(trait_upcasting)]

use dada_ir_ast::{ast::Identifier, diagnostic::Diagnostic, inputs::SourceFile};
use dada_ir_sym::{
    ir::{functions::SymFunction, types::SymGenericTerm},
    prelude::Symbol,
    Db,
};

mod cx;

#[salsa::tracked(return_ref)]
pub fn codegen_main_fn<'db>(db: &'db dyn Db, source_file: SourceFile) -> Option<Vec<u8>> {
    let main = Identifier::main(db);
    let module = source_file.symbol(db);
    let main_fn = module.function_named(db, main)?;

    if !main_fn.symbols(db).has_generics_of_kind(db, &[]) {
        let error = Diagnostic::error(
            db,
            main_fn.name_span(db),
            "main function must have no generics",
        );
        error.report(db);
        return None;
    }

    Some(codegen(db, main_fn, vec![]).clone())
}

/// Generate a self-contained wasm module from a starting function.
#[salsa::tracked(return_ref)]
pub fn codegen<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
    generics: Vec<SymGenericTerm<'db>>,
) -> Vec<u8> {
    cx::Cx::new(db)
        .generate_from_fn(function, generics)
        .finish()
}
