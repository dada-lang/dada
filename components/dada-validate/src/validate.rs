use dada_ir::code::validated;
use dada_ir::code::Code;
use dada_ir::filename::Filename;
use dada_parse::prelude::*;

use self::name_lookup::Scope;

mod name_lookup;
mod validator;

/// Computes a validated tree for the given code (may produce errors).
#[salsa::memoized(in crate::Jar)]
pub fn validate_code(db: &dyn crate::Db, code: Code) -> validated::Tree {
    let syntax_tree = code.syntax_tree(db);
    let mut tables = validated::Tables::default();
    let mut origins = validated::Origins::default();
    let root_definitions = root_definitions(db, code.filename(db));
    let scope = Scope::root(db, root_definitions);
    let mut validator =
        validator::Validator::new(db, code, syntax_tree, &mut tables, &mut origins, scope);
    let root_expr = validator.validate_expr(syntax_tree.root_expr);
    let data = validated::TreeData::new(tables, root_expr);
    validated::Tree::new(db, code, data)
}

/// Compute the root definitions for the module. This is not memoized to
/// save effort but rather because it may generate errors and we don't want to issue those
/// errors multiple times.
#[salsa::memoized(in crate::Jar ref)]
pub fn root_definitions(db: &dyn crate::Db, filename: Filename) -> name_lookup::RootDefinitions {
    name_lookup::RootDefinitions::new(db, filename)
}
