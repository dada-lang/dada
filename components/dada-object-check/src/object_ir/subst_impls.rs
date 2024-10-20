use dada_ir_sym::{
    indices::SymBinderIndex,
    subst::{subst_var, Subst, SubstGenericVar, SubstWith},
    ty::Var,
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

    fn subst_with(
        &self,
        db: &'db dyn dada_ir_sym::Db,
        start_binder: SymBinderIndex,
        subst_fns: &mut dada_ir_sym::subst::SubstitutionFns<'_, 'db, ObjectGenericTerm<'db>>,
    ) -> Self::Output {
        match self.kind(db) {
            ObjectTyKind::Named(sym_ty_name, vec) => todo!(),
            ObjectTyKind::Var(var) => subst_var(db, start_binder, subst_fns, self, *var),
            ObjectTyKind::Error(_) => self.identity(),
            ObjectTyKind::Never => self.identity(),
        }
    }
}

impl<'db> SubstGenericVar<'db> for ObjectTy<'db> {
    fn assert_kind(db: &'db dyn crate::Db, term: ObjectGenericTerm<'db>) -> Self {
        term.assert_type(db)
    }

    fn bound_var(
        db: &'db dyn dada_ir_sym::Db,
        binder_index: SymBinderIndex,
        bound_var_index: dada_ir_sym::indices::SymBoundVarIndex,
    ) -> Self {
        ObjectTy::new(
            db,
            ObjectTyKind::Var(Var::Bound(binder_index, bound_var_index)),
        )
    }
}
