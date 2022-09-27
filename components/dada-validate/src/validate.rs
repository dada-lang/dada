use dada_ir::code::validated;
use dada_ir::function::{Function, FunctionSignature};
use dada_ir::input_file::InputFile;
use dada_parse::prelude::*;

use crate::name_lookup::{RootDefinitions, Scope};

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
            validator.validate_signature_into_tree(s);
        }
        FunctionSignature::Main => {}
    }
    let num_parameters = validator.num_local_variables();

    let root_expr = validator.validate_root_expr(syntax_tree.data(db).root_expr);
    std::mem::drop(validator);
    let data = validated::TreeData::new(tables, num_parameters, root_expr);
    validated::Tree::new(db, function, data, origins)
}

/// Compute the root definitions for the module. This is not memoized to
/// save effort but rather because it may generate errors and we don't want to issue those
/// errors multiple times.
#[salsa::tracked(return_ref)]
#[allow(clippy::needless_lifetimes)]
pub fn root_definitions(db: &dyn crate::Db, input_file: InputFile) -> RootDefinitions {
    RootDefinitions::new(db, input_file)
}
