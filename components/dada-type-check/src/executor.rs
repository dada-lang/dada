use std::{error::Report, ops::Deref};

use dada_ir_ast::{diagnostic::Reported, span::Span};
use dada_ir_sym::{
    indices::SymVarIndex,
    symbol::SymGenericKind,
    ty::{GenericIndex, SymGenericTerm, SymTy},
};
use typed_arena::Arena;

use crate::{
    checking_ir::{Expr, ExprKind, PlaceExpr, PlaceExprKind},
    env::Env,
    inference::InferenceVarData,
    universe::Universe,
};

pub(crate) struct Check<'chk, 'db> {
    pub db: &'db dyn crate::Db,
    arenas: &'chk ExecutorArenas<'chk, 'db>,
    deferred: Vec<DeferredCheck<'chk, 'db>>,
    inference_vars: Vec<InferenceVarData<'db>>,
}

#[derive(Default)]
pub struct ExecutorArenas<'chk, 'db> {
    expr_kinds: Arena<ExprKind<'chk, 'db>>,
    place_expr_kinds: Arena<PlaceExprKind<'chk, 'db>>,
}

struct DeferredCheck<'chk, 'db> {
    env: Env<'db>,
    thunk: Box<dyn FnOnce(&mut Check<'chk, 'db>, Env<'db>) + 'chk>,
}

impl<'chk, 'db> Check<'chk, 'db> {
    pub fn new(db: &'db dyn crate::Db, arenas: &'chk ExecutorArenas<'chk, 'db>) -> Self {
        Self {
            db,
            arenas,
            inference_vars: Vec::new(),
            deferred: Vec::new(),
        }
    }

    pub fn unit(&self) -> SymTy<'db> {
        SymTy::unit(self.db)
    }

    /// Allocate an expression
    pub fn expr(
        &mut self,
        span: Span<'db>,
        ty: SymTy<'db>,
        kind: ExprKind<'chk, 'db>,
    ) -> Expr<'chk, 'db> {
        let kind = self.arenas.expr_kinds.alloc(kind);
        Expr { span, ty, kind }
    }

    pub fn err_expr(&mut self, span: Span<'db>, reported: Reported) -> Expr<'chk, 'db> {
        self.expr(span, self.unit(), ExprKind::Error(reported))
    }

    /// Allocate a place expression
    pub fn place_expr(
        &mut self,
        span: Span<'db>,
        ty: SymTy<'db>,
        kind: PlaceExprKind<'chk, 'db>,
    ) -> PlaceExpr<'chk, 'db> {
        let kind = self.arenas.place_expr_kinds.alloc(kind);
        PlaceExpr { span, ty, kind }
    }

    /// Create a series of semi-colon separated expressions.
    /// The final result type will be the type of the last expression.
    /// Returns `None` if exprs is empty.
    pub fn exprs(
        &mut self,
        exprs: impl IntoIterator<Item = Expr<'chk, 'db>>,
    ) -> Option<Expr<'chk, 'db>> {
        let mut lhs: Option<Expr<'_, '_>> = None;
        for rhs in exprs {
            lhs = Some(match lhs {
                None => rhs,
                Some(result) => self.expr(
                    result.span.to(rhs.span),
                    rhs.ty,
                    ExprKind::Semi(result, rhs),
                ),
            });
        }

        lhs
    }

    pub fn fresh_inference_var(
        &mut self,
        kind: SymGenericKind,
        universe: Universe,
    ) -> SymGenericTerm<'db> {
        let var_index = SymVarIndex::from(self.inference_vars.len());
        self.inference_vars
            .push(InferenceVarData::new(kind, universe));

        SymGenericTerm::var(self.db, kind, GenericIndex::Existential(var_index))
    }

    pub fn defer_check(
        &mut self,
        env: &Env<'db>,
        chk: impl FnOnce(&mut Check<'chk, 'db>, Env<'db>) + 'chk,
    ) {
        self.deferred.push(DeferredCheck {
            env: env.clone(),
            thunk: Box::new(chk),
        });
    }
}

impl<'db> Deref for Check<'_, 'db> {
    type Target = &'db dyn crate::Db;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}
