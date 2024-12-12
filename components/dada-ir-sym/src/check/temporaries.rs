use dada_ir_ast::span::Span;

use crate::ir::{
    exprs::{SymExpr, SymPlaceExpr, SymPlaceExprKind},
    types::{SymGenericKind, SymTy},
    variables::SymVariable,
};

/// Translating an expression can result in the creation of
/// anonymous local temporaries that are injected into the
/// surrounding scope. These are returned alongside the result
/// and will eventually be translated into `let-in` expressions
/// when we reach the surrounding statement, block, or other
/// terminating context.
#[derive(Clone)]
pub(crate) struct Temporary<'db> {
    pub lv: SymVariable<'db>,
    pub ty: SymTy<'db>,
    pub initializer: Option<SymExpr<'db>>,
}

impl<'db> Temporary<'db> {
    pub fn new(
        db: &'db dyn crate::Db,
        span: Span<'db>,
        ty: SymTy<'db>,
        initializer: Option<SymExpr<'db>>,
    ) -> Self {
        let lv = SymVariable::new(db, SymGenericKind::Place, None, span);
        Self {
            lv,
            ty,
            initializer,
        }
    }
}

impl<'db> SymExpr<'db> {
    pub(crate) fn into_temporary(
        self,
        db: &'db dyn crate::Db,
        temporaries: &mut Vec<Temporary<'db>>,
    ) -> SymPlaceExpr<'db> {
        let ty = self.ty(db);

        // Create a temporary to store the result of this expression.
        let temporary = Temporary::new(db, self.span(db), self.ty(db), Some(self));
        let lv = temporary.lv;
        temporaries.push(temporary);

        // The result will be a reference to that temporary.
        SymPlaceExpr::new(db, self.span(db), ty, SymPlaceExprKind::Var(lv))
    }
}
