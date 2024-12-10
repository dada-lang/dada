use dada_ir_ast::diagnostic::Err;
use dada_ir_sym::{
    function::{SymFunction, SymInputOutput},
    prelude::{CheckedBody, CheckedSignature},
    symbol::SymVariable,
    ty::{SymGenericTerm, SymTy},
};
use dada_util::Map;
use wasm_encoder::ValType;

use super::{generate_expr::ExprCodegen, Cx, FnIndex, FnKey};

impl<'db> Cx<'db> {
    /// Declares an instantiation of a function with a given set of arguments and returns its index.
    /// If the function is already declared, nothing happens.
    /// If the function is not already declared, it is enqueued for code-generation.
    pub(crate) fn declare_fn(
        &mut self,
        function: SymFunction<'db>,
        generics: Vec<SymGenericTerm<'db>>,
    ) -> FnIndex {
        let key = FnKey(function, generics);
        let generics: &Vec<SymGenericTerm<'_>> = &key.1;

        // Check if we already declared this function and return the result if so
        if let Some(index) = self.functions.get(&key).copied() {
            return index;
        }

        // Extract function signature
        let CodegenSignature {
            inputs: _,
            generics: _,
            input_output:
                SymInputOutput {
                    input_tys,
                    output_ty,
                },
        } = self.codegen_signature(function, &generics);

        // Create the type for this function
        let ty_index = {
            // The first input is the stack pointer.
            // The remainder are the values given by the user.
            let input_val_types = std::iter::once(ValType::I32)
                .chain(
                    input_tys
                        .iter()
                        .flat_map(|&t| self.wasm_repr_of_type(t, &Default::default()).flatten()),
                )
                .collect::<Vec<_>>();
            let output_val_types = self
                .wasm_repr_of_type(output_ty, &Default::default())
                .flatten();
            self.declare_fn_type(input_val_types, output_val_types)
        };

        // Add to the WASM function section
        let fn_index =
            FnIndex(u32::try_from(self.function_section.len()).expect("too many functions"));
        self.function_section.function(u32::from(ty_index));

        // Record on the queue to generate code
        self.codegen_queue.push(key.clone().into());

        // Memoize the result for later
        self.functions.insert(key, fn_index);

        fn_index
    }

    pub(crate) fn codegen_fn(&mut self, FnKey(function, generics): FnKey<'db>) {
        let db = self.db;

        let object_check_body = match function.checked_body(self.db) {
            Some(body) => body,
            None => panic!("asked to codegen function with no body: {function:?}"),
        };

        let CodegenSignature {
            inputs,
            generics,
            input_output,
        } = self.codegen_signature(function, &generics);

        // Generate the function body.
        let function = {
            let mut ecx = ExprCodegen::new(self, generics);
            ecx.pop_arguments(inputs, &input_output.input_tys);
            ecx.push_expr(object_check_body);
            ecx.pop_and_return(object_check_body.ty(db));
            ecx.into_function()
        };

        self.code_section.function(&function);
    }

    fn codegen_signature(
        &self,
        function: SymFunction<'db>,
        generics: &[SymGenericTerm<'db>],
    ) -> CodegenSignature<'db> {
        match function.checked_signature(self.db) {
            Ok(signature) => {
                let symbols = signature.symbols(self.db);

                let input_output = signature
                    .input_output(self.db)
                    .substitute(self.db, generics);
                let dummy_places = symbols
                    .input_variables
                    .iter()
                    .map(|_| SymGenericTerm::Place(todo!()))
                    .collect::<Vec<_>>();
                let input_output = input_output.substitute(self.db, &dummy_places);

                CodegenSignature {
                    inputs: &symbols.input_variables,
                    generics: symbols
                        .generic_variables
                        .iter()
                        .copied()
                        .zip(generics.iter().copied())
                        .collect(),
                    input_output,
                }
            }

            Err(reported) => CodegenSignature {
                inputs: &[],
                generics: Default::default(),
                input_output: SymInputOutput {
                    input_tys: vec![],
                    output_ty: SymTy::err(self.db, reported),
                },
            },
        }
    }
}

struct CodegenSignature<'db> {
    inputs: &'db [SymVariable<'db>],
    generics: Map<SymVariable<'db>, SymGenericTerm<'db>>,
    input_output: SymInputOutput<'db>,
}
