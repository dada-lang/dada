//! Type checking orchestration for Dada programs.
#![doc = include_str!("../docs/overview.md")]

use dada_ir_ast::{
    ast::Identifier,
    diagnostic::{Diagnostic, Level},
    inputs::SourceFile,
    span::Spanned,
};
use dada_ir_sym::{
    ir::{
        binder::{Binder, BoundTerm},
        classes::{SymAggregate, SymClassMember, SymField},
        functions::{SignatureSymbols, SymFunction, SymFunctionSignature, SymInputOutput},
        generics::{SymWhereClause, SymWhereClauseKind},
        module::{SymItem, SymModule},
        types::{SymGenericKind, SymGenericTerm, SymPerm, SymPlace, SymTy},
        variables::SymVariable,
    },
    prelude::*,
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
        self.symbol(db).check(db);
    }
}

impl<'db> Check<'db> for SymModule<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        self.items(db).for_each(|item| item.check(db));
        self.check_use_items(db);
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
        self.checked_field_ty(db);
    }
}

impl<'db> Check<'db> for SymFunction<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        let _ = self.checked_signature(db);
        self.checked_body(db);
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
            format!("duplicate parameter name `{id}`"),
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
            where_clauses,
        } = self;
        input_tys.check(db);
        output_ty.check(db);
        where_clauses.check(db);
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

impl<'db> Check<'db> for SymWhereClause<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        self.subject(db).check(db);
        self.kind(db).check(db);
    }
}

impl<'db> Check<'db> for SymWhereClauseKind {
    fn check(&self, _db: &'db dyn crate::Db) {
        match self {
            SymWhereClauseKind::Unique => (),
            SymWhereClauseKind::Shared => (),
            SymWhereClauseKind::Owned => (),
            SymWhereClauseKind::Lent => (),
        }
    }
}

impl<'db> Check<'db> for SymGenericTerm<'db> {
    fn check(&self, db: &'db dyn crate::Db) {
        match self {
            SymGenericTerm::Type(sym_ty) => sym_ty.check(db),
            SymGenericTerm::Perm(sym_perm) => sym_perm.check(db),
            SymGenericTerm::Place(sym_place) => sym_place.check(db),
            SymGenericTerm::Error(_) => (),
        }
    }
}

impl<'db> Check<'db> for SymPerm<'db> {
    fn check(&self, _db: &'db dyn crate::Db) {
        // There *are* validity checks that need to be done on permissions,
        // but they are done as part of the checking the item in which
        // the permission appears.
    }
}

impl<'db> Check<'db> for SymPlace<'db> {
    fn check(&self, _db: &'db dyn crate::Db) {
        // There *are* validity checks that need to be done on places,
        // but they are done as part of the checking the item in which
        // the place appears.
    }
}
