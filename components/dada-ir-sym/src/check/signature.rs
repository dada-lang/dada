use dada_ir_ast::{
    ast::AstFunctionInput,
    diagnostic::{Diagnostic, Err, Errors},
    span::Spanned,
};
use dada_util::Map;

use crate::{
    check::env::{Env, EnvLike},
    check::runtime::Runtime,
    check::scope::Scope,
    check::CheckInEnv,
    ir::functions::{SymFunction, SymFunctionSignature, SymFunctionSource, SymInputOutput},
    ir::types::{SymTy, SymTyName},
    ir::variables::SymVariable,
    prelude::Symbol,
};

pub fn check_function_signature<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
) -> Errors<SymFunctionSignature<'db>> {
    Runtime::execute(
        db,
        function.name_span(db),
        async move |runtime| -> Errors<SymFunctionSignature<'db>> {
            let (env, _, input_output) = prepare_env(db, runtime, function);

            let scope = env.into_scope();
            Ok(SymFunctionSignature::new(
                db,
                function.symbols(db).clone(),
                scope.into_bound_value(db, input_output),
            ))
        },
    )
}

pub fn prepare_env<'db>(
    db: &'db dyn crate::Db,
    runtime: &Runtime<'db>,
    function: SymFunction<'db>,
) -> (Env<'db>, Vec<SymVariable<'db>>, SymInputOutput<'db>) {
    let source = function.source(db);
    let inputs = source.inputs(db);
    let input_symbols = inputs
        .iter()
        .map(|input| input.symbol(db))
        .collect::<Vec<_>>();
    let mut proto_env = ProtoEnv {
        db,
        scope: function.scope(db),
        inputs: input_symbols.iter().copied().zip(inputs.iter()).collect(),
        input_tys: Default::default(),
        stack: Default::default(),
    };

    let mut env: Env<'db> = Env::new(runtime, function.scope(db));
    let mut input_tys: Vec<SymTy<'db>> = vec![];
    for i in source.inputs(db).iter() {
        let ty = proto_env.variable_ty(i.symbol(db));
        env.set_program_variable_ty(i.symbol(db), ty);
        input_tys.push(ty);
    }

    let mut output_ty: SymTy<'db> = output_ty(&mut proto_env, &function);
    if function.effects(db).async_effect {
        output_ty = SymTy::named(db, SymTyName::Future, vec![output_ty.into()]);
    }
    env.set_return_ty(output_ty);

    (
        env,
        input_symbols,
        SymInputOutput {
            input_tys,
            output_ty,
        },
    )
}

struct ProtoEnv<'a, 'db> {
    db: &'db dyn crate::Db,
    scope: Scope<'db, 'db>,
    inputs: Map<SymVariable<'db>, &'a AstFunctionInput<'db>>,
    input_tys: Map<SymVariable<'db>, SymTy<'db>>,
    stack: Vec<SymVariable<'db>>,
}

impl<'a, 'db> EnvLike<'db> for ProtoEnv<'a, 'db> {
    fn db(&self) -> &'db dyn crate::Db {
        self.db
    }

    fn scope(&self) -> &Scope<'db, 'db> {
        &self.scope
    }

    fn variable_ty(&mut self, lv: SymVariable<'db>) -> SymTy<'db> {
        if let Some(&ty) = self.input_tys.get(&lv) {
            return ty;
        }

        let input = self.inputs[&lv];
        if self.stack.contains(&lv) {
            return SymTy::err(
                self.db,
                Diagnostic::error(
                    self.db,
                    input.span(self.db),
                    format!("type of `{lv}` references itself"),
                )
                .report(self.db),
            );
        }

        self.stack.push(lv);
        self.variable_ty_from_input(input)
    }
}

impl<'a, 'db> ProtoEnv<'a, 'db> {
    fn variable_ty_from_input(&mut self, input: &AstFunctionInput<'db>) -> SymTy<'db> {
        match input {
            AstFunctionInput::SelfArg(arg) => {
                if let Some(aggregate) = self.scope.class() {
                    let self_ty = aggregate.self_ty(self.db, &self.scope);
                    match arg.perm(self.db) {
                        Some(ast_perm) => {
                            let sym_perm = ast_perm.check_in_env(self);
                            SymTy::perm(self.db, sym_perm, self_ty)
                        }
                        None => self_ty,
                    }
                } else {
                    SymTy::err(
                        self.db,
                        Diagnostic::error(
                            self.db,
                            arg.span(self.db),
                            "self parameter is only permitted within a class definition",
                        )
                        .report(self.db),
                    )
                }
            }
            AstFunctionInput::Variable(var) => var.ty(self.db()).check_in_env(self),
        }
    }
}

fn output_ty<'a, 'db>(env: &mut ProtoEnv<'a, 'db>, function: &SymFunction<'db>) -> SymTy<'db> {
    let db = env.db();
    match function.source(db) {
        SymFunctionSource::Function(ast_function) => match ast_function.output_ty(db) {
            Some(ast_ty) => ast_ty.check_in_env(env),
            None => SymTy::unit(db),
        },
        SymFunctionSource::Constructor(sym_aggregate, _ast_aggregate) => {
            sym_aggregate.self_ty(db, &env.scope)
        }
    }
}
