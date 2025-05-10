use dada_ir_ast::{
    ast::AstFunctionInput,
    diagnostic::{Diagnostic, Err, Errors},
    span::Spanned,
};

use crate::{
    check::{env::Env, runtime::Runtime},
    ir::{
        functions::{SymFunction, SymFunctionSignature, SymFunctionSource, SymInputOutput},
        types::{SymTy, SymTyName},
        variables::SymVariable,
    },
    prelude::Symbol,
};

use super::CheckTyInEnv;

pub fn check_function_signature<'db>(
    db: &'db dyn crate::Db,
    function: SymFunction<'db>,
) -> Errors<SymFunctionSignature<'db>> {
    Runtime::execute(
        db,
        function.name_span(db),
        async move |runtime| -> Errors<SymFunctionSignature<'db>> {
            let PreparedEnv {
                env,
                input_tys,
                output_ty_caller,
                ..
            } = prepare_env(db, runtime, function).await;

            let scope = env.into_scope();
            Ok(SymFunctionSignature::new(
                db,
                function.symbols(db).clone(),
                scope.into_bound_value(
                    db,
                    SymInputOutput {
                        input_tys,
                        output_ty: output_ty_caller,
                    },
                ),
            ))
        },
        |v| v,
    )
}

pub struct PreparedEnv<'db> {
    /// The env that should be used to type check the body
    pub env: Env<'db>,

    /// The generic variables declared on the fn
    pub input_symbols: Vec<SymVariable<'db>>,

    /// The types of the fn inputs
    pub input_tys: Vec<SymTy<'db>>,

    /// The return type the block should generate.
    /// This is the type that the user wrote.
    /// In the case of an async fn, this is not a future.
    pub output_ty_body: SymTy<'db>,

    /// The return type of the fn from the perspective of the caller.
    /// For an async fn, this is a future.
    pub output_ty_caller: SymTy<'db>,
}

pub async fn prepare_env<'db>(
    db: &'db dyn crate::Db,
    runtime: &Runtime<'db>,
    function: SymFunction<'db>,
) -> PreparedEnv<'db> {
    let source = function.source(db);
    let inputs = source.inputs(db);
    let input_symbols = inputs
        .iter()
        .map(|input| input.symbol(db))
        .collect::<Vec<_>>();

    let mut env: Env<'db> = Env::new(runtime, function.scope(db));

    // Set the AST types for the inputs.
    for i in source.inputs(db).iter() {
        set_variable_ty_from_input(&mut env, i).await;
    }

    // Now that all input types are available, symbolify and create `input_tys` vector.
    let mut input_tys: Vec<SymTy<'db>> = vec![];
    for i in source.inputs(db).iter() {
        let ty = env.variable_ty(i.symbol(db)).await;
        input_tys.push(ty);
    }

    // Symbolify the output type.
    let output_ty_body: SymTy<'db> = output_ty(&mut env, &function).await;
    env.set_return_ty(output_ty_body);

    let output_ty_caller = if function.effects(db).async_effect {
        SymTy::named(db, SymTyName::Future, vec![output_ty_body.into()])
    } else {
        output_ty_body
    };

    PreparedEnv {
        env,
        input_symbols,
        input_tys,
        output_ty_body,
        output_ty_caller,
    }
}

async fn set_variable_ty_from_input<'db>(env: &mut Env<'db>, input: &AstFunctionInput<'db>) {
    let db = env.db();
    let lv = input.symbol(db);
    match input {
        AstFunctionInput::SelfArg(arg) => {
            let self_ty = if let Some(aggregate) = env.scope.aggregate() {
                let aggr_ty = aggregate.self_ty(db, &env.scope);
                let ast_perm = arg.perm(db);
                let sym_perm = ast_perm.check_in_env(env).await;
                SymTy::perm(db, sym_perm, aggr_ty)
            } else {
                SymTy::err(
                    db,
                    Diagnostic::error(
                        db,
                        arg.span(db),
                        "self parameter is only permitted within a class definition",
                    )
                    .report(db),
                )
            };
            env.set_variable_sym_ty(lv, self_ty);
        }
        AstFunctionInput::Variable(var) => {
            env.set_variable_ast_ty(lv, var.ty(db));
        }
    }
}

async fn output_ty<'db>(env: &mut Env<'db>, function: &SymFunction<'db>) -> SymTy<'db> {
    let db = env.db();
    match function.source(db) {
        SymFunctionSource::Function(ast_function) => match ast_function.output_ty(db) {
            Some(ast_ty) => ast_ty.check_in_env(env).await,
            None => SymTy::unit(db),
        },
        SymFunctionSource::Constructor(sym_aggregate, _ast_aggregate) => {
            sym_aggregate.self_ty(db, &env.scope)
        }
    }
}
