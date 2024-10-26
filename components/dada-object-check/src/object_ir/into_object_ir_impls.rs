use dada_ir_sym::{
    binder::{Binder, BoundTerm},
    ty::{SymGenericTerm, SymTy, SymTyKind},
};

use super::{IntoObjectIr, ObjectGenericTerm, ObjectTy, ObjectTyKind};

impl<'db> IntoObjectIr<'db> for ObjectTy<'db> {
    type Object = Self;

    fn into_object_ir(self, _db: &'db dyn crate::Db) -> ObjectTy<'db> {
        self
    }
}

impl<'db> IntoObjectIr<'db> for SymTy<'db> {
    type Object = ObjectTy<'db>;

    fn into_object_ir(self, db: &'db dyn crate::Db) -> ObjectTy<'db> {
        match self.kind(db) {
            SymTyKind::Perm(_, ty) => ty.into_object_ir(db),
            SymTyKind::Named(name, vec) => ObjectTy::new(
                db,
                ObjectTyKind::Named(*name, vec.iter().map(|t| t.into_object_ir(db)).collect()),
            ),
            SymTyKind::Var(var) => ObjectTy::new(db, ObjectTyKind::Var(*var)),
            SymTyKind::Error(reported) => ObjectTy::new(db, ObjectTyKind::Error(*reported)),
            SymTyKind::Never => ObjectTy::new(db, ObjectTyKind::Never),
            SymTyKind::Infer(sym_infer_var_index) => {
                ObjectTy::new(db, ObjectTyKind::Infer(*sym_infer_var_index))
            }
        }
    }
}

impl<'db> IntoObjectIr<'db> for SymGenericTerm<'db> {
    type Object = ObjectGenericTerm<'db>;

    fn into_object_ir(self, db: &'db dyn crate::Db) -> ObjectGenericTerm<'db> {
        match self {
            SymGenericTerm::Type(ty) => ObjectGenericTerm::Type(ty.into_object_ir(db)),
            SymGenericTerm::Perm(_) => ObjectGenericTerm::Perm,
            SymGenericTerm::Error(reported) => ObjectGenericTerm::Error(reported),
            SymGenericTerm::Place(_) => ObjectGenericTerm::Place,
        }
    }
}

impl<'db, T> IntoObjectIr<'db> for Binder<'db, T>
where
    T: BoundTerm<'db> + IntoObjectIr<'db, Object: BoundTerm<'db>>,
{
    type Object = Binder<'db, T::Object>;

    fn into_object_ir(self, db: &'db dyn crate::Db) -> Self::Object {
        self.map(db, |t| t.into_object_ir(db))
    }
}
