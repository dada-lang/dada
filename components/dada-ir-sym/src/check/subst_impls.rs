use dada_ir_ast::ast::PermissionOp;

use crate::ir::{
    exprs::{
        SymBinaryOp, SymByteLiteral, SymExpr, SymExprKind, SymLiteral, SymMatchArm, SymPlaceExpr,
        SymPlaceExprKind,
    },
    functions::SymFunctionSignature,
    subst::{Subst, SubstWith, SubstitutionFns, identity_subst},
    types::SymGenericTerm,
    variables::SymVariable,
};

impl<'db> Subst<'db> for SymExpr<'db> {
    type GenericTerm = SymGenericTerm<'db>;
}

impl<'db> SubstWith<'db, SymGenericTerm<'db>> for SymExpr<'db> {
    type Output = SymExpr<'db>;

    fn identity(&self) -> Self::Output {
        *self
    }

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
    ) -> Self::Output {
        let span = self.span(db);
        let ty = self.ty(db).subst_with(db, bound_vars, subst_fns);
        let kind = self.kind(db).subst_with(db, bound_vars, subst_fns);
        SymExpr::new(db, span, ty, kind)
    }
}

impl<'db> Subst<'db> for SymExprKind<'db> {
    type GenericTerm = SymGenericTerm<'db>;
}

impl<'db> SubstWith<'db, SymGenericTerm<'db>> for SymExprKind<'db> {
    type Output = SymExprKind<'db>;

    fn identity(&self) -> Self::Output {
        self.clone()
    }

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
    ) -> Self::Output {
        match self {
            SymExprKind::Semi(sym_expr, sym_expr1) => SymExprKind::Semi(
                sym_expr.subst_with(db, bound_vars, subst_fns),
                sym_expr1.subst_with(db, bound_vars, subst_fns),
            ),
            SymExprKind::Tuple(vec) => {
                SymExprKind::Tuple(vec.subst_with(db, bound_vars, subst_fns))
            }
            SymExprKind::Primitive(sym_literal) => {
                SymExprKind::Primitive(sym_literal.subst_with(db, bound_vars, subst_fns))
            }
            SymExprKind::ByteLiteral(sym_byte_literal) => {
                SymExprKind::ByteLiteral(sym_byte_literal.subst_with(db, bound_vars, subst_fns))
            }
            SymExprKind::LetIn {
                lv,
                ty,
                initializer,
                body,
            } => SymExprKind::LetIn {
                lv: *lv,
                ty: ty.subst_with(db, bound_vars, subst_fns),
                initializer: initializer.subst_with(db, bound_vars, subst_fns),
                body: bind_variable(*lv, bound_vars, |bound_vars| {
                    body.subst_with(db, bound_vars, subst_fns)
                }),
            },
            SymExprKind::Await {
                future,
                await_keyword,
            } => SymExprKind::Await {
                future: future.subst_with(db, bound_vars, subst_fns),
                await_keyword: await_keyword.subst_with(db, bound_vars, subst_fns),
            },
            SymExprKind::Assign { place, value } => SymExprKind::Assign {
                place: place.subst_with(db, bound_vars, subst_fns),
                value: value.subst_with(db, bound_vars, subst_fns),
            },
            SymExprKind::PermissionOp(permission_op, sym_place_expr) => SymExprKind::PermissionOp(
                permission_op.subst_with(db, bound_vars, subst_fns),
                sym_place_expr.subst_with(db, bound_vars, subst_fns),
            ),
            SymExprKind::Call {
                function,
                substitution,
                arg_temps,
            } => SymExprKind::Call {
                function: function.subst_with(db, bound_vars, subst_fns),
                substitution: substitution.subst_with(db, bound_vars, subst_fns),
                arg_temps: arg_temps
                    .iter()
                    .map(|&t| assert_bound_variable(db, t, bound_vars))
                    .collect(),
            },
            SymExprKind::Return(sym_expr) => {
                SymExprKind::Return(sym_expr.subst_with(db, bound_vars, subst_fns))
            }
            SymExprKind::Not { operand, op_span } => SymExprKind::Not {
                operand: operand.subst_with(db, bound_vars, subst_fns),
                op_span: op_span.subst_with(db, bound_vars, subst_fns),
            },
            SymExprKind::BinaryOp(sym_binary_op, sym_expr, sym_expr1) => SymExprKind::BinaryOp(
                sym_binary_op.subst_with(db, bound_vars, subst_fns),
                sym_expr.subst_with(db, bound_vars, subst_fns),
                sym_expr1.subst_with(db, bound_vars, subst_fns),
            ),
            SymExprKind::Aggregate { ty, fields } => SymExprKind::Aggregate {
                ty: ty.subst_with(db, bound_vars, subst_fns),
                fields: fields.subst_with(db, bound_vars, subst_fns),
            },
            SymExprKind::Match { arms } => SymExprKind::Match {
                arms: arms.subst_with(db, bound_vars, subst_fns),
            },
            SymExprKind::Error(reported) => {
                SymExprKind::Error(reported.subst_with(db, bound_vars, subst_fns))
            }
        }
    }
}

impl<'db> Subst<'db> for SymPlaceExpr<'db> {
    type GenericTerm = SymGenericTerm<'db>;
}

impl<'db> SubstWith<'db, SymGenericTerm<'db>> for SymPlaceExpr<'db> {
    type Output = SymPlaceExpr<'db>;

    fn identity(&self) -> Self::Output {
        *self
    }

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
    ) -> Self::Output {
        SymPlaceExpr::new(
            db,
            self.span(db).subst_with(db, bound_vars, subst_fns),
            self.ty(db).subst_with(db, bound_vars, subst_fns),
            self.kind(db).subst_with(db, bound_vars, subst_fns),
        )
    }
}

impl<'db> SubstWith<'db, SymGenericTerm<'db>> for SymPlaceExprKind<'db> {
    type Output = SymPlaceExprKind<'db>;

    fn identity(&self) -> Self::Output {
        self.clone()
    }

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
    ) -> Self::Output {
        match *self {
            SymPlaceExprKind::Var(sym_variable) => {
                SymPlaceExprKind::Var(assert_bound_variable(db, sym_variable, bound_vars))
            }
            SymPlaceExprKind::Field(sym_place_expr, sym_field) => SymPlaceExprKind::Field(
                sym_place_expr.subst_with(db, bound_vars, subst_fns),
                sym_field.subst_with(db, bound_vars, subst_fns),
            ),
            SymPlaceExprKind::Error(reported) => {
                SymPlaceExprKind::Error(reported.subst_with(db, bound_vars, subst_fns))
            }
        }
    }
}

impl<'db> Subst<'db> for SymMatchArm<'db> {
    type GenericTerm = SymGenericTerm<'db>;
}

impl<'db> SubstWith<'db, SymGenericTerm<'db>> for SymMatchArm<'db> {
    type Output = SymMatchArm<'db>;

    fn identity(&self) -> Self::Output {
        self.clone()
    }

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
    ) -> Self::Output {
        let SymMatchArm { condition, body } = self;
        SymMatchArm {
            condition: condition.subst_with(db, bound_vars, subst_fns),
            body: body.subst_with(db, bound_vars, subst_fns),
        }
    }
}

impl<'db> Subst<'db> for SymFunctionSignature<'db> {
    type GenericTerm = SymGenericTerm<'db>;
}

impl<'db> SubstWith<'db, SymGenericTerm<'db>> for SymFunctionSignature<'db> {
    type Output = SymFunctionSignature<'db>;

    fn identity(&self) -> Self::Output {
        *self
    }

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<SymVariable<'db>>,
        subst_fns: &mut SubstitutionFns<'_, 'db, SymGenericTerm<'db>>,
    ) -> Self::Output {
        let symbols = self.symbols(db);
        let len = bound_vars.len();
        bound_vars.extend_from_slice(&symbols.generic_variables);
        bound_vars.extend_from_slice(&symbols.input_variables);
        let input_output = self.input_output(db).subst_with(db, bound_vars, subst_fns);
        bound_vars.truncate(len);
        SymFunctionSignature::new(db, symbols.clone(), input_output)
    }
}

identity_subst! {
    for 'db {
        SymBinaryOp,
        PermissionOp,
        SymLiteral,
        SymByteLiteral<'db>,
    }
}

fn bind_variable<'db, T>(
    sym_variable: SymVariable<'db>,
    bound_vars: &mut Vec<SymVariable<'db>>,
    op: impl FnOnce(&mut Vec<SymVariable<'db>>) -> T,
) -> T {
    bound_vars.push(sym_variable);
    let result = op(bound_vars);
    bound_vars.pop().unwrap();
    result
}

fn assert_bound_variable<'db>(
    _db: &'db dyn crate::Db,
    sym_variable: SymVariable<'db>,
    bound_vars: &mut Vec<SymVariable<'db>>,
) -> SymVariable<'db> {
    // Program variables should always appear bound, never free, and hence are never substituted.
    assert!(bound_vars.contains(&sym_variable));
    sym_variable
}
