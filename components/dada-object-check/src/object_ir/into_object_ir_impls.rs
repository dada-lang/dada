use dada_ir_sym::{
    binder::{Binder, BoundTerm},
    function::SymInputOutput,
    primitive::SymPrimitive,
    ty::{SymGenericTerm, SymTy, SymTyKind},
};

use crate::env::Env;

use super::{ObjectGenericTerm, ObjectInputOutput, ObjectTy, ObjectTyKind, ToObjectIr};

impl<'db> ToObjectIr<'db> for ObjectTy<'db> {
    type Object = Self;

    fn to_object_ir(&self, _env: &Env<'db>) -> ObjectTy<'db> {
        *self
    }
}

impl<'db> ToObjectIr<'db> for SymTy<'db> {
    type Object = ObjectTy<'db>;

    fn to_object_ir(&self, env: &Env<'db>) -> ObjectTy<'db> {
        let db = env.db();
        match self.kind(db) {
            SymTyKind::Perm(_, ty) => ty.to_object_ir(env),
            SymTyKind::Named(name, vec) => ObjectTy::new(
                db,
                ObjectTyKind::Named(*name, vec.iter().map(|t| t.to_object_ir(env)).collect()),
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

impl<'db> ToObjectIr<'db> for SymGenericTerm<'db> {
    type Object = ObjectGenericTerm<'db>;

    fn to_object_ir(&self, env: &Env<'db>) -> ObjectGenericTerm<'db> {
        match self {
            SymGenericTerm::Type(ty) => ObjectGenericTerm::Type(ty.to_object_ir(env)),
            SymGenericTerm::Perm(_) => ObjectGenericTerm::Perm,
            SymGenericTerm::Error(reported) => ObjectGenericTerm::Error(*reported),
            SymGenericTerm::Place(_) => ObjectGenericTerm::Place,
        }
    }
}

impl<'db, T> ToObjectIr<'db> for Binder<'db, T>
where
    T: BoundTerm<'db> + ToObjectIr<'db, Object: BoundTerm<'db>>,
{
    type Object = Binder<'db, T::Object>;

    fn to_object_ir(&self, env: &Env<'db>) -> Self::Object {
        let db = env.db();
        self.map_ref(db, |t| t.to_object_ir(env))
    }
}

impl<'db> ToObjectIr<'db> for SymPrimitive<'db> {
    type Object = ObjectTy<'db>;

    fn to_object_ir(&self, env: &Env<'db>) -> ObjectTy<'db> {
        let db = env.db();
        ObjectTy::new(db, ObjectTyKind::Named((*self).into(), vec![]))
    }
}

impl<'db> ToObjectIr<'db> for SymInputOutput<'db> {
    type Object = ObjectInputOutput<'db>;

    fn to_object_ir(&self, env: &Env<'db>) -> Self::Object {
        ObjectInputOutput {
            input_tys: self.input_tys.to_object_ir(env),
            output_ty: self.output_ty.to_object_ir(env),
        }
    }
}

impl<'db, T> ToObjectIr<'db> for Vec<T>
where
    T: ToObjectIr<'db>,
{
    type Object = Vec<T::Object>;

    fn to_object_ir(&self, env: &Env<'db>) -> Self::Object {
        self.iter().map(|t| t.to_object_ir(env)).collect()
    }
}
