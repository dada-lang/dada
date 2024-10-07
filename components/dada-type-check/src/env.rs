use dada_ir_sym::{scope::Scope, symbol::SymLocalVariable, ty::SymTy};
use dada_util::Map;

pub struct Env<'env, 'db> {
    pub db: &'db dyn crate::Db,
    pub scope: &'env Scope<'env, 'db>,
    pub program_variables: Map<SymLocalVariable<'db>, SymTy<'db>>,
}

impl<'env, 'db> Env<'env, 'db> {
    /// Create an empty environment
    pub fn new(db: &'db dyn crate::Db, scope: &'env Scope<'env, 'db>) -> Self {
        Self {
            db,
            scope,
            program_variables: Map::default(),
        }
    }

    pub fn insert_program_variable(&mut self, lv: SymLocalVariable<'db>, ty: SymTy<'db>) {
        self.program_variables.insert(lv, ty);
    }
}
