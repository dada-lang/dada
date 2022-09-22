use dada_id::InternKey;
use dada_ir::class::Class;
use dada_ir::code::{syntax, validated};
use dada_ir::function::{Function, FunctionSignature};
use dada_ir::input_file::InputFile;
use dada_ir::parameter::Parameter;
use dada_ir::storage::Atomic;
use dada_parse::prelude::*;

use self::name_lookup::Scope;

mod name_lookup;
mod validator;

/// Computes a validated tree for the given code (may produce errors).
#[salsa::tracked]
#[tracing::instrument(level = "debug", skip(db))]
pub(crate) fn validate_function(db: &dyn crate::Db, function: Function) -> validated::Tree {
    let syntax_tree = function.syntax_tree(db);

    let mut tables = validated::Tables::default();
    let mut origins = validated::Origins::default();
    let root_definitions = root_definitions(db, function.input_file(db));
    let scope = Scope::root(db, root_definitions);

    let mut validator =
        validator::Validator::root(db, function, syntax_tree, &mut tables, &mut origins, scope);

    match function.signature(db) {
        FunctionSignature::Syntax(s) => {
            validator.validate_signature(s);
        }
        FunctionSignature::Main => {}
    }
    let num_parameters = validator.num_local_variables();

    let root_expr = validator.validate_root_expr(syntax_tree.data(db).root_expr);
    std::mem::drop(validator);
    let data = validated::TreeData::new(tables, num_parameters, root_expr);
    validated::Tree::new(db, function, data, origins)
}

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
    signature_parameters(db, class.signature(db))
}

fn signature_parameters(db: &dyn crate::Db, signature: &syntax::Signature) -> Vec<Parameter> {
    let tables = &signature.tables;
    signature
        .parameters
        .iter()
        .map(|&lv| {
            let lv_data = lv.data(tables);
            let atomic = match lv_data.atomic {
                Some(_) => Atomic::Yes,
                None => Atomic::No,
            };
            let name = lv_data.name.data(tables).word;
            Parameter::new(db, name, atomic)
        })
        .collect()
}

/// Compute the root definitions for the module. This is not memoized to
/// save effort but rather because it may generate errors and we don't want to issue those
/// errors multiple times.
#[salsa::tracked(return_ref)]
#[allow(clippy::needless_lifetimes)]
pub fn root_definitions(db: &dyn crate::Db, input_file: InputFile) -> name_lookup::RootDefinitions {
    name_lookup::RootDefinitions::new(db, input_file)
}
