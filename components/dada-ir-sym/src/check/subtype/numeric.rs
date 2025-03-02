use dada_ir_ast::diagnostic::Errors;

use crate::{
    check::{
        chains::{RedTy, ToRedTy},
        env::Env,
        report::{Because, OrElse},
    },
    ir::{
        primitive::SymPrimitiveKind,
        types::{SymTy, SymTyKind, SymTyName},
    },
};

pub async fn require_numeric_type<'db>(
    env: &Env<'db>,
    mut ty: SymTy<'db>,
    or_else: &dyn OrElse<'db>,
) -> Errors<()> {
    let db = env.db();
    loop {
        let mut red_ty = ty.to_red_ty(db);
        match red_ty {
            RedTy::Error(reported) => return Err(reported),
            RedTy::Named(sym_ty_name, _) => match sym_ty_name {
                SymTyName::Primitive(sym_primitive) => match sym_primitive.kind(db) {
                    SymPrimitiveKind::Bool | SymPrimitiveKind::Char => {
                        return Err(or_else.report(db, Because::NotNumeric(red_ty)));
                    }
                    SymPrimitiveKind::Int { bits: _ }
                    | SymPrimitiveKind::Isize
                    | SymPrimitiveKind::Uint { bits: _ }
                    | SymPrimitiveKind::Usize
                    | SymPrimitiveKind::Float { bits: _ } => return Ok(()),
                },
                SymTyName::Aggregate(_) | SymTyName::Future | SymTyName::Tuple { arity: _ } => {
                    return Err(or_else.report(db, Because::NotNumeric(red_ty)));
                }
            },

            RedTy::Var(_) | RedTy::Never => {
                return Err(or_else.report(db, Because::NotNumeric(red_ty)));
            }

            RedTy::Infer(infer_var_index) => {
                todo!()
            }

            RedTy::Perm => unreachable!("SymTy had a red ty of SymPerm"),
        }
    }
}
