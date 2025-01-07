use crate::ir::{
    exprs::{SymExpr, SymExprKind, SymMatchArm, SymPlaceExpr, SymPlaceExprKind},
    subst,
    types::{SymGenericTerm, SymPlace},
};

use super::{
    Env,
    resolve::{Resolver, Variance},
};

pub(super) trait ResolveInferenceVariables<'db> {
    fn resolve_inference_vars_in(&self, env: &Env<'db>) -> Self;
}

impl<'db> ResolveInferenceVariables<'db> for SymExpr<'db> {
    fn resolve_inference_vars_in(&self, env: &Env<'db>) -> SymExpr<'db> {
        let db = env.db();
        let ty = Resolver::new(env).resolve_term(self.ty(db), Variance::Covariant);
        let span = self.span(db);
        let kind = match *self.kind(db) {
            SymExprKind::Semi(l, r) => SymExprKind::Semi(
                l.resolve_inference_vars_in(env),
                r.resolve_inference_vars_in(env),
            ),
            SymExprKind::Tuple(ref vec) => SymExprKind::Tuple(vec.resolve_inference_vars_in(env)),
            SymExprKind::Primitive(sym_literal) => SymExprKind::Primitive(sym_literal),
            SymExprKind::ByteLiteral(sym_byte_literal) => {
                SymExprKind::ByteLiteral(sym_byte_literal)
            }
            SymExprKind::LetIn {
                lv,
                ty,
                initializer,
                body,
            } => SymExprKind::LetIn {
                lv,
                ty: Resolver::new(env).resolve_term(ty, Variance::Covariant),
                initializer: initializer.resolve_inference_vars_in(env),
                body: body.resolve_inference_vars_in(env),
            },
            SymExprKind::Await {
                future,
                await_keyword,
            } => SymExprKind::Await {
                future: future.resolve_inference_vars_in(env),
                await_keyword: await_keyword,
            },
            SymExprKind::Assign { place, value } => SymExprKind::Assign {
                place: place.resolve_inference_vars_in(env),
                value: value.resolve_inference_vars_in(env),
            },
            SymExprKind::PermissionOp(permission_op, sym_place_expr) => SymExprKind::PermissionOp(
                permission_op,
                sym_place_expr.resolve_inference_vars_in(env),
            ),
            SymExprKind::Call {
                function,
                ref substitution,
                ref arg_temps,
            } => SymExprKind::Call {
                function: function,
                substitution: Resolver::new(env).resolve_term(substitution, Variance::Invariant),
                arg_temps: arg_temps.clone(),
            },
            SymExprKind::Return(sym_expr) => {
                SymExprKind::Return(sym_expr.resolve_inference_vars_in(env))
            }
            SymExprKind::Not { operand, op_span } => SymExprKind::Not {
                operand: operand.resolve_inference_vars_in(env),
                op_span,
            },
            SymExprKind::BinaryOp(sym_binary_op, sym_expr, sym_expr1) => SymExprKind::BinaryOp(
                sym_binary_op,
                sym_expr.resolve_inference_vars_in(env),
                sym_expr1.resolve_inference_vars_in(env),
            ),
            SymExprKind::Aggregate { ty, ref fields } => SymExprKind::Aggregate {
                ty: Resolver::new(env).resolve_term(ty, Variance::Covariant),
                fields: fields.resolve_inference_vars_in(env),
            },
            SymExprKind::Match { ref arms } => SymExprKind::Match {
                arms: arms.resolve_inference_vars_in(env),
            },
            SymExprKind::Error(reported) => SymExprKind::Error(reported),
        };
        SymExpr::new(db, span, ty, kind)
    }
}

impl<'db> ResolveInferenceVariables<'db> for SymMatchArm<'db> {
    fn resolve_inference_vars_in(&self, env: &Env<'db>) -> Self {
        let db = env.db();
        let SymMatchArm { condition, body } = self;
        let condition = condition.resolve_inference_vars_in(env);
        let body = body.resolve_inference_vars_in(env);
        SymMatchArm { condition, body }
    }
}

impl<'db> ResolveInferenceVariables<'db> for SymPlaceExpr<'db> {
    fn resolve_inference_vars_in(&self, env: &Env<'db>) -> Self {
        let db = env.db();
        let ty = Resolver::new(env).resolve_term(self.ty(db), Variance::Covariant);
        let span = self.span(db);
        let kind = match self.kind(db) {
            SymPlaceExprKind::Var(sym_variable) => SymPlaceExprKind::Var(*sym_variable),
            SymPlaceExprKind::Field(sym_place_expr, sym_field_name) => SymPlaceExprKind::Field(
                sym_place_expr.resolve_inference_vars_in(env),
                *sym_field_name,
            ),
            SymPlaceExprKind::Error(reported) => SymPlaceExprKind::Error(*reported),
        };
        SymPlaceExpr::new(db, span, ty, kind)
    }
}

impl<'db, T> ResolveInferenceVariables<'db> for Vec<T>
where
    T: ResolveInferenceVariables<'db>,
{
    fn resolve_inference_vars_in(&self, env: &Env<'db>) -> Vec<T> {
        self.iter()
            .map(|e| e.resolve_inference_vars_in(env))
            .collect()
    }
}

impl<'db, T> ResolveInferenceVariables<'db> for Option<T>
where
    T: ResolveInferenceVariables<'db>,
{
    fn resolve_inference_vars_in(&self, env: &Env<'db>) -> Option<T> {
        self.as_ref().map(|e| e.resolve_inference_vars_in(env))
    }
}
