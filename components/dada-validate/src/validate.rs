use dada_ir::code::validated;
use dada_ir::code::Code;
use dada_parse::prelude::*;

use self::name_lookup::Scope;

mod name_lookup;
mod validator;

#[salsa::memoized(in crate::Jar ref)]
pub fn validate_code(db: &dyn crate::Db, code: Code) -> validated::Tree {
    let syntax_tree = code.syntax_tree(db);
    let mut tables = validated::Tables::default();
    let mut origins = validated::Origins::default();
    let scope = Scope::root(db, code.filename(db));
    let mut validator =
        validator::Validator::new(db, code, syntax_tree, &mut tables, &mut origins, scope);
    let root_expr = validator.validate_expr(syntax_tree.root_expr);
    validated::Tree::new(tables, root_expr)
}
