use std::{cell::RefCell, future::Future, pin::Pin, sync::Arc};

use dada_ir_ast::{diagnostic::Reported, span::Span};
use dada_ir_sym::{
    indices::SymVarIndex,
    symbol::SymGenericKind,
    ty::{GenericIndex, SymGenericTerm, SymTy},
};
use dada_util::Map;
use futures::FutureExt;
use typed_arena::Arena;

use crate::{
    checking_ir::{Expr, ExprKind, PlaceExpr, PlaceExprKind},
    env::Env,
    inference::InferenceVarData,
    universe::Universe,
};

type Deferred<'chk> = Pin<Box<dyn Future<Output = ()> + 'chk>>;

#[derive(Clone)]
pub(crate) struct Check<'chk, 'db> {
    data: Arc<CheckData<'chk, 'db>>,
}

pub(crate) struct CheckData<'chk, 'db> {
    pub db: &'db dyn crate::Db,
    arenas: &'chk ExecutorArenas<'chk, 'db>,
    inference_vars: RefCell<Vec<InferenceVarData<'db>>>,
    ready_to_execute: RefCell<Vec<Deferred<'chk>>>,
    waiting_on_inference_var: RefCell<Map<SymVarIndex, Vec<Deferred<'chk>>>>,
}

impl<'chk, 'db> std::ops::Deref for Check<'chk, 'db> {
    type Target = CheckData<'chk, 'db>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Default)]
pub struct ExecutorArenas<'chk, 'db> {
    expr_kinds: Arena<ExprKind<'chk, 'db>>,
    place_expr_kinds: Arena<PlaceExprKind<'chk, 'db>>,
}

struct DeferredCheck<'chk, 'db> {
    env: Env<'db>,
    thunk: Box<dyn FnOnce(&Check<'chk, 'db>, Env<'db>) + 'chk>,
}

impl<'chk, 'db> Check<'chk, 'db> {
    pub fn new(db: &'db dyn crate::Db, arenas: &'chk ExecutorArenas<'chk, 'db>) -> Self {
        Self {
            data: Arc::new(CheckData {
                db,
                arenas,
                inference_vars: Default::default(),
                ready_to_execute: Default::default(),
                waiting_on_inference_var: Default::default(),
            }),
        }
    }

    pub fn unit(&self) -> SymTy<'db> {
        SymTy::unit(self.db)
    }

    /// Allocate an expression
    pub fn expr(
        &self,
        span: Span<'db>,
        ty: SymTy<'db>,
        kind: ExprKind<'chk, 'db>,
    ) -> Expr<'chk, 'db> {
        let kind = self.arenas.expr_kinds.alloc(kind);
        Expr { span, ty, kind }
    }

    pub fn err_expr(&self, span: Span<'db>, reported: Reported) -> Expr<'chk, 'db> {
        self.expr(span, self.unit(), ExprKind::Error(reported))
    }

    /// Allocate a place expression
    pub fn place_expr(
        &self,
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
        &self,
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
        &self,
        kind: SymGenericKind,
        universe: Universe,
    ) -> SymGenericTerm<'db> {
        let mut inference_vars = self.inference_vars.borrow_mut();
        let var_index = SymVarIndex::from(inference_vars.len());
        inference_vars.push(InferenceVarData::new(kind, universe));
        SymGenericTerm::var(self.db, kind, GenericIndex::Existential(var_index))
    }

    /// Execute the given future asynchronously from the main execution.
    /// It must execute to completion eventually or an error will be reported.
    pub fn defer<F>(&self, env: &Env<'db>, thunk: impl FnOnce(Check<'chk, 'db>, Env<'db>) -> F)
    where
        F: Future<Output = ()> + 'chk,
    {
        let future = thunk(self.clone(), env.clone());
        self.ready_to_execute
            .borrow_mut()
            .push(future.boxed_local());
    }
}
