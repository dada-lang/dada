use dada_ir_ast::span::Span;
use dada_ir_sym::{
    indices::SymVarIndex,
    scope::Scope,
    symbol::SymGenericKind,
    ty::{GenericIndex, SymGenericTerm, SymTy, SymTyKind},
};
use typed_arena::Arena;

use crate::{
    checking_ir::{Expr, ExprKind},
    env::Env,
    inference::InferenceVarData,
    universe::{self, Universe},
};

pub(crate) struct Check<'chk, 'db> {
    pub db: &'db dyn crate::Db,
    pub scope: Scope<'db, 'db>,
    arenas: &'chk ExecutorArenas<'chk, 'db>,
    deferred: Vec<DeferredCheck<'chk, 'db>>,
    inference_vars: Vec<InferenceVarData<'db>>,
}

#[derive(Default)]
pub struct ExecutorArenas<'chk, 'db> {
    kinds: Arena<ExprKind<'chk, 'db>>,
}

struct DeferredCheck<'chk, 'db> {
    env: Env<'db>,
    thunk: Box<dyn FnOnce(&mut Check<'chk, 'db>, &Env<'db>) + 'chk>,
}

impl<'chk, 'db> Check<'chk, 'db> {
    pub fn new(
        db: &'db dyn crate::Db,
        scope: Scope<'db, 'db>,
        arenas: &'chk ExecutorArenas<'chk, 'db>,
    ) -> Self {
        Self {
            db,
            scope,
            arenas,
            inference_vars: Vec::new(),
            deferred: Vec::new(),
        }
    }

    /// Allocate an expression
    pub fn expr(
        &mut self,
        span: Span<'db>,
        ty: SymTy<'db>,
        kind: ExprKind<'chk, 'db>,
    ) -> Expr<'chk, 'db> {
        let kind = self.arenas.kinds.alloc(kind);
        Expr { span, ty, kind }
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
        chk: impl FnOnce(&mut Check<'chk, 'db>, &Env<'db>) + 'chk,
    ) {
        self.deferred.push(DeferredCheck {
            env: env.clone(),
            thunk: Box::new(chk),
        });
    }
}
