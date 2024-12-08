#![feature(trait_upcasting)]

use dada_ir_ast::{
    ast::Identifier,
    diagnostic::{Diagnostic, Level},
    inputs::SourceFile,
    span::Spanned,
};
use dada_ir_sym::{
    binder::{Binder, BoundTerm},
    class::{SymAggregate, SymClassMember, SymField},
    function::{SignatureSymbols, SymFunction, SymFunctionSignature, SymInputOutput},
    module::{SymItem, SymModule},
    prelude::*,
    symbol::{SymGenericKind, SymVariable},
    ty::SymTy,
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

impl<'db, T: Check<'db>> Check<'db> for Option<T> {
    fn check(&self, db: &'db dyn crate::Db) {
        if let Some(t) = self {
            t.check(db);
        }
    }
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

impl<'db> Check<'db> for SymAggregate<'db> {
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
        self.object_check_body(db).check(db);
    }
}

impl<'db> Check<'db> for SymFunctionSignature<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        self.symbols(db).check(db);

        self.input_output(db).check(db);
    }
}

impl<'db> Check<'db> for SignatureSymbols<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        let mut variable_names = Map::default();
        for &variable in self.generic_variables.iter().chain(&self.input_variables) {
            variable.check(db);

            if let Some(id) = variable.name(db) {
                check_for_duplicates(db, &mut variable_names, id, variable);
            }
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

impl<'db> Check<'db> for SymInputOutput<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        let SymInputOutput {
            input_tys,
            output_ty,
        } = self;
        input_tys.check(db);
        output_ty.check(db);
    }
}

impl<'db, C: Check<'db>> Check<'db> for Vec<C> {
    fn check(&self, db: &'db dyn crate::Db) {
        for item in self {
            item.check(db);
        }
    }
}

impl<'db> Check<'db> for SymTy<'db> {
    fn check(&self, _db: &'db dyn crate::Db) {
        // There *are* validity checks that need to be done on types,
        // but they are done as part of the checking the item in which
        // the type appears.
    }
}

impl<'db, C: Check<'db> + BoundTerm<'db>> Check<'db> for Binder<'db, C> {
    fn check(&self, db: &'db dyn crate::Db) {
        for sym in &self.variables {
            sym.check(db);
        }
        self.bound_value.check(db);
    }
}

impl<'db> Check<'db> for SymGenericKind {
    fn check(&self, _db: &'db dyn crate::Db) {}
}

impl<'db> Check<'db> for SymVariable<'db> {
    fn check(&self, _db: &'db dyn crate::Db) {
        // There *are* validity checks that need to be done on types,
        // but they are done as part of the checking the item in which
        // the type appears.
    }
}

impl<'db> Check<'db> for ObjectExpr<'db> {
    fn check(&self, _db: &'db dyn crate::Db) {
        // FIXME: true check-check
    }
}
