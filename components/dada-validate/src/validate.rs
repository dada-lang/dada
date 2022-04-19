use dada_ir::code::validated;
use dada_ir::filename::Filename;
use dada_ir::function::Function;
use dada_parse::prelude::*;

use self::name_lookup::Scope;

mod name_lookup;
mod validator;

/// Computes a validated tree for the given code (may produce errors).
#[salsa::memoized(in crate::Jar)]
#[tracing::instrument(level = "debug", skip(db))]
pub fn validate_function(db: &dyn crate::Db, function: Function) -> validated::Tree {
    let code = function.code(db);
    let syntax_tree = code.syntax_tree(db);

    let mut tables = validated::Tables::default();
    let mut origins = validated::Origins::default();
    let root_definitions = root_definitions(db, code.filename(db));
    let scope = Scope::root(db, root_definitions);

    let mut validator = validator::Validator::new(
        db,
        code,
        syntax_tree,
        &mut tables,
        &mut origins,
        scope,
        |_| function.effect_span(db),
    );

    for parameter in &syntax_tree.data(db).parameter_decls {
        validator.validate_parameter(*parameter);
    }
    let num_parameters = validator.num_local_variables();

    let root_expr = validator.give_validated_root_expr(syntax_tree.data(db).root_expr);
    std::mem::drop(validator);
    let data = validated::TreeData::new(tables, num_parameters, root_expr);
    validated::Tree::new(db, function, data, origins)
}

/// Compute the root definitions for the module. This is not memoized to
/// save effort but rather because it may generate errors and we don't want to issue those
/// errors multiple times.
#[salsa::memoized(in crate::Jar ref)]
#[allow(clippy::needless_lifetimes)]
pub fn root_definitions(db: &dyn crate::Db, filename: Filename) -> name_lookup::RootDefinitions {
    name_lookup::RootDefinitions::new(db, filename)
}
