use crate::prelude::*;
use dada_ir::code::validated;
use dada_ir::function::Function;
use dada_ir::input_file::InputFile;
use dada_ir::parameter::Parameter;
use dada_parse::prelude::*;

use self::name_lookup::Scope;

mod name_lookup;
mod validator;

/// Computes a validated tree for the given code (may produce errors).
#[salsa::tracked]
#[tracing::instrument(level = "debug", skip(db))]
pub(crate) fn validate_function(db: &dyn crate::Db, function: Function) -> validated::Tree {
    let parameters = function.parameters(db);
    let syntax_tree = function.syntax_tree(db);

    let mut tables = validated::Tables::default();
    let mut origins = validated::Origins::default();
    let root_definitions = root_definitions(db, function.input_file(db));
    let scope = Scope::root(db, root_definitions);

    let mut validator =
        validator::Validator::root(db, function, syntax_tree, &mut tables, &mut origins, scope);

    for parameter in parameters {
        validator.validate_parameter(*parameter);
    }
    let num_parameters = validator.num_local_variables();

    let root_expr = validator.validate_root_expr(syntax_tree.data(db).root_expr);
    std::mem::drop(validator);
    let data = validated::TreeData::new(tables, num_parameters, root_expr);
    validated::Tree::new(db, function, data, origins)
}

#[salsa::tracked(return_ref)]
pub(crate) fn validate_parameter(db: &dyn crate::Db, function: Function) -> Vec<Parameter> {
    return function._parameters(db).clone();
}

/// Compute the root definitions for the module. This is not memoized to
/// save effort but rather because it may generate errors and we don't want to issue those
/// errors multiple times.
#[salsa::tracked(return_ref)]
#[allow(clippy::needless_lifetimes)]
pub fn root_definitions(db: &dyn crate::Db, input_file: InputFile) -> name_lookup::RootDefinitions {
    name_lookup::RootDefinitions::new(db, input_file)
}
