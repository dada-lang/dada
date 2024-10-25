use dada_ir_ast::diagnostic::Err;
use dada_ir_sym::{
    subst::{subst_var, Subst, SubstWith, SubstitutionFns},
    symbol::{AssertKind, HasKind, SymGenericKind, SymVariable},
};

use super::{ObjectGenericTerm, ObjectTy, ObjectTyKind};

impl<'db> Subst<'db> for ObjectTy<'db> {
    type GenericTerm = ObjectGenericTerm<'db>;
}

impl<'db> SubstWith<'db, ObjectGenericTerm<'db>> for ObjectTy<'db> {
    type Output = Self;

    fn identity(&self) -> Self::Output {
        *self
    }

    fn subst_with<'subst>(
        &'subst self,
        db: &'db dyn crate::Db,
        bound_vars: &mut Vec<&'subst [SymVariable<'db>]>,
        subst_fns: &mut SubstitutionFns<'_, 'db, ObjectGenericTerm<'db>>,
    ) -> Self::Output {
        match self.kind(db) {
            ObjectTyKind::Named(sym_ty_name, vec) => todo!(),
            ObjectTyKind::Var(var) => subst_var(db, bound_vars, subst_fns, *var),
            ObjectTyKind::Error(_) => self.identity(),
            ObjectTyKind::Never => self.identity(),
            ObjectTyKind::Infer(_) => self.identity(),
        }
    }
}

impl<'db> AssertKind<'db, ObjectTy<'db>> for ObjectGenericTerm<'db> {
    fn assert_kind(self, db: &'db dyn dada_ir_sym::Db) -> ObjectTy<'db> {
        assert!(self.has_kind(db, SymGenericKind::Type));
        match self {
            ObjectGenericTerm::Type(ty) => ty,
            ObjectGenericTerm::Error(r) => ObjectTy::err(db, r),
            _ => unreachable!(),
        }
    }
}
