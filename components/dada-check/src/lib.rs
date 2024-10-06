#![feature(trait_upcasting)]

use dada_ir_ast::{
    ast::Identifier,
    diagnostic::{Diagnostic, Level},
    inputs::SourceFile,
    span::Spanned,
};
use dada_ir_sym::{
    class::{SymClass, SymClassMember, SymField},
    function::{SignatureSymbols, SymFunction, SymFunctionSignature},
    module::{SymItem, SymModule},
    prelude::*,
    symbol::{SymGeneric, SymLocalVariable},
    ty::{Binder, SymTy},
};

pub use dada_ir_sym::Db;
use dada_util::Map;

pub mod prelude {
    pub use crate::Check;
}

/// The main "check" routine. This defines what it means for a dada program to successfully compile.
pub trait Check<'db> {
    fn check(&self, db: &'db dyn crate::Db);
}

impl<'db> Check<'db> for SourceFile {
    fn check(&self, db: &'db dyn crate::Db) {
        self.into_symbol(db).check(db);
    }
}

impl<'db> Check<'db> for SymModule<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        self.items(db).for_each(|item| item.check(db));
        self.resolve_use_items(db);
    }
}

impl<'db> Check<'db> for SymItem<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        match self {
            SymItem::SymClass(sym_class) => sym_class.check(db),
            SymItem::SymFunction(sym_function) => sym_function.check(db),
            SymItem::SymPrimitive(_sym_primtive) => (),
        }
    }
}

impl<'db> Check<'db> for SymClass<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        self.members(db).iter().for_each(|member| member.check(db));
    }
}

impl<'db> Check<'db> for SymClassMember<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        match self {
            SymClassMember::SymField(sym_field) => sym_field.check(db),
            SymClassMember::SymFunction(sym_function) => sym_function.check(db),
        }
    }
}

impl<'db> Check<'db> for SymField<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        self.ty(db).check(db);
    }
}

impl<'db> Check<'db> for SymFunction<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        self.signature(db).check(db);
    }
}

impl<'db> Check<'db> for SymFunctionSignature<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        self.symbols(db).check(db);

        for input_ty in self.input_tys(db) {
            input_ty.check(db);
        }

        self.output_ty(db).check(db);
    }
}

impl<'db> Check<'db> for SignatureSymbols<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        let mut generic_names = Map::default();
        for generic in &self.generics {
            generic.check(db);

            if let Some(id) = generic.name(db) {
                check_for_duplicates(db, &mut generic_names, id, *generic);
            }
        }

        let mut input_names = Map::default();
        for input in &self.inputs {
            input.check(db);
            check_for_duplicates(db, &mut input_names, input.name(db), *input);
        }
    }
}

fn check_for_duplicates<'db, S: Spanned<'db>>(
    db: &'db dyn crate::Db,
    map: &mut Map<Identifier<'db>, S>,
    id: Identifier<'db>,
    value: S,
) {
    if let Some(other_input) = map.get(&id) {
        Diagnostic::error(
            db,
            value.span(db),
            format!("duplicate parameter name `{}`", id),
        )
        .label(
            db,
            Level::Error,
            value.span(db),
            "all parameters must have unique names, but this parameter has the same name as another parameter",
        )
        .label(
            db,
            Level::Info,
            other_input.span(db),
            "the previous parameter is here",
        )
        .report(db);
    }

    map.insert(id, value);
}

impl<'db> Check<'db> for SymTy<'db> {
    fn check(&self, _db: &'db dyn crate::Db) {
        // There *are* validity checks that need to be done on types,
        // but they are done as part of the checking the item in which
        // the type appears.
    }
}

impl<'db, C: Check<'db>> Check<'db> for Binder<'db, C> {
    fn check(&self, db: &'db dyn crate::Db) {
        for sym in &self.symbols {
            sym.check(db);
        }
        self.bound_value.check(db);
    }
}

impl<'db> Check<'db> for SymGeneric<'db> {
    fn check(&self, _db: &'db dyn crate::Db) {
        // There *are* validity checks that need to be done on types,
        // but they are done as part of the checking the item in which
        // the type appears.
    }
}

impl<'db> Check<'db> for SymLocalVariable<'db> {
    fn check(&self, _db: &'db dyn crate::Db) {
        // There *are* validity checks that need to be done on types,
        // but they are done as part of the checking the item in which
        // the type appears.
    }
}
