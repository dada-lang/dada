#![feature(trait_upcasting)]

use dada_ir_sym::{function::SymFunction, ty::SymGenericTerm, Db};

mod cx;

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
