use dada_ir_ast::{
    ast::Identifier,
    diagnostic::{Errors, Reported},
    inputs::Krate,
};

use crate::{
    class::SymAggregate,
    module::{SymItem, SymModule},
    IntoSymbol,
};

fn dada_lang_krate(db: &dyn crate::Db) -> Errors<Krate> {
    let root = db.root();
    match root.libdada_crate(db) {
        Some(krate) => Ok(krate),
        None => Err(report_not_found(db, "the `dada` crate")),
    }
}

fn prelude_module<'db>(db: &'db dyn crate::Db) -> Errors<SymModule<'db>> {
    let krate = dada_lang_krate(db)?;
    let identifier = Identifier::new(db, "prelude");
    Ok(db.source_file(krate, &[identifier]).into_symbol(db))
}

fn prelude_member<'db>(db: &'db dyn crate::Db, name: &str) -> Errors<SymItem<'db>> {
    let identifier = Identifier::new(db, name);
    let module = prelude_module(db)?;
    module
        .items(db)
        .find(|item| item.name(db) == identifier)
        .ok_or_else(|| report_not_found(db, &format!("`{name}` in the `libdada` prelude")))
}

#[salsa::tracked]
pub fn string_class<'db>(db: &'db dyn crate::Db) -> Errors<SymAggregate<'db>> {
    match prelude_member(db, "String")? {
        SymItem::SymClass(class) => {
            if !class.symbols(db).generic_variables.is_empty() {
                return Err(report_unexpected(db, "String", "it has generic parameters"));
            }
            Ok(class)
        }
        _ => Err(report_unexpected(db, "String", "it is not a class")),
    }
}

fn report_not_found<'db>(_db: &'db dyn crate::Db, name: &str) -> Reported {
    // TODO: figure out how to report a diagnostic with some kind of default span
    panic!("could not find {name}")
}

fn report_unexpected<'db>(_db: &'db dyn crate::Db, name: &str, problem: &str) -> Reported {
    // TODO: figure out how to report a diagnostic with some kind of default span
    panic!("found {name} but {problem}")
}
