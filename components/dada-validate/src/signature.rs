use dada_id::InternKey;
use dada_ir::class::Class;
use dada_ir::code::syntax;
use dada_ir::function::{Function, FunctionSignature};
use dada_ir::signature::Parameter;
use dada_ir::storage::Atomic;

#[salsa::tracked(return_ref)]
pub(crate) fn validate_function_parameters(
    db: &dyn crate::Db,
    function: Function,
) -> Vec<Parameter> {
    match function.signature(db) {
        FunctionSignature::Main => vec![],

        FunctionSignature::Syntax(s) => signature_parameters(db, s),
    }
}

#[salsa::tracked(return_ref)]
pub(crate) fn validate_class_fields(db: &dyn crate::Db, class: Class) -> Vec<Parameter> {
    let signature = class.signature(db);
    signature_parameters(db, signature)
}

fn signature_parameters(db: &dyn crate::Db, signature: &syntax::Signature) -> Vec<Parameter> {
    let tables = &signature.tables;
    signature
        .parameters
        .iter()
        .map(|&lv| {
            let lv_data = lv.data(tables);
            let name = lv_data.name.data(tables).word;
            let atomic = Atomic::from(lv_data.atomic);
            Parameter::new(db, name, None, atomic)
        })
        .collect()
}
