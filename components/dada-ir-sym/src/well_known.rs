use dada_ir_ast::{
    ast::Identifier,
    diagnostic::{Diagnostic, Errors, Reported},
    span::{Span, Spanned},
};

use crate::{
    ir::{
        classes::{SymAggregate, SymClassMember},
        functions::SymFunction,
        module::{SymItem, SymModule},
        types::SymGenericKind,
    },
    prelude::Symbol,
};

/// Returns the span of the `libdada` prelude. Used when a span is needed for diagnostics
/// for something built-in, like a primitive.
pub fn prelude_span<'db>(db: &'db dyn crate::Db) -> Span<'db> {
    prelude_module(db).span(db)
}

/// Returns the `libdada` prelude module. It must be present.
fn prelude_module<'db>(db: &'db dyn crate::Db) -> SymModule<'db> {
    let krate = db.root().libdada_crate(db);
    let identifier = Identifier::prelude(db);
    db.source_file(krate, &[identifier]).symbol(db)
}

/// Returns the member of the `libdada` prelude with the given name,
/// reporting an error if it is not found.
fn prelude_member<'db>(db: &'db dyn crate::Db, name: &str) -> Errors<SymItem<'db>> {
    let identifier = Identifier::new(db, name);
    let module = prelude_module(db);
    module
        .items(db)
        .find(|item| item.name(db) == identifier)
        .ok_or_else(|| report_not_found(db, module, &format!("`{name}` in the `libdada` prelude")))
}

/// Returns the `String` class from the `libdada` prelude.
#[salsa::tracked]
pub fn string_class<'db>(db: &'db dyn crate::Db) -> Errors<SymAggregate<'db>> {
    match prelude_member(db, "String")? {
        SymItem::SymClass(class) if class.is_class(db) => {
            if !class.symbols(db).has_generics_of_kind(db, &[]) {
                return Err(report_unexpected(
                    db,
                    class,
                    "String",
                    "it has generic parameters",
                ));
            }
            Ok(class)
        }
        m => Err(report_unexpected(db, m, "String", "it is not a class")),
    }
}

/// Returns the `literal` function of the `String` class from the `libdada` prelude.
#[salsa::tracked]
pub fn string_literal_fn<'db>(db: &'db dyn crate::Db) -> Errors<SymFunction<'db>> {
    let string_class = string_class(db)?;
    let literal_fn = string_class
        .inherent_member_str(db, "literal")
        .ok_or_else(|| {
            report_unexpected(
                db,
                string_class,
                "String",
                "does not have a `literal` member",
            )
        })?;
    match literal_fn {
        SymClassMember::SymFunction(function) => {
            if !function.symbols(db).has_generics_of_kind(db, &[]) {
                return Err(report_unexpected(
                    db,
                    function,
                    "String",
                    "`literal` should not have generic parameters",
                ));
            }
            Ok(function)
        }
        m => Err(report_unexpected(
            db,
            m,
            "String",
            "`literal` is not a function",
        )),
    }
}

/// Returns the `Pointer` struct from the `libdada` prelude.
#[salsa::tracked]
pub fn pointer_struct<'db>(db: &'db dyn crate::Db) -> Errors<SymAggregate<'db>> {
    match prelude_member(db, "Pointer")? {
        SymItem::SymClass(class) if class.is_struct(db) => {
            if !class
                .symbols(db)
                .has_generics_of_kind(db, &[SymGenericKind::Type])
            {
                return Err(report_unexpected(
                    db,
                    class,
                    "Pointer",
                    "it should have 1 generic parameter",
                ));
            }
            Ok(class)
        }
        m => Err(report_unexpected(db, m, "Pointer", "it is not a struct")),
    }
}

fn report_not_found<'db>(db: &'db dyn crate::Db, module: SymModule<'db>, name: &str) -> Reported {
    let module_span = module.span(db);
    Diagnostic::error(db, module_span, format!("could not find {name}")).report(db)
}

fn report_unexpected<'db>(
    db: &'db dyn crate::Db,
    spanned: impl Spanned<'db>,
    name: &str,
    problem: &str,
) -> Reported {
    let span = spanned.span(db);
    Diagnostic::error(db, span, format!("found {name} but {problem}")).report(db)
}
