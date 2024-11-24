use dada_ir_sym::function::SymFunction;
use dada_object_check::object_ir::ObjectGenericTerm;
use dada_util::{FromImpls, Map};
use salsa::Update;
use wasm_encoder::{CodeSection, FunctionSection, TypeSection};

mod generate_expr;
mod generate_fn;
mod wasm_repr;

/// Core codegen context.
pub(crate) struct Cx<'db> {
    db: &'db dyn crate::Db,
    function_section: FunctionSection,
    type_section: TypeSection,
    code_section: CodeSection,
    functions: Map<FnKey<'db>, FnIndex>,
    codegen_queue: Vec<CodegenQueueItem<'db>>,
}

impl<'db> Cx<'db> {
    pub fn new(db: &'db dyn crate::Db) -> Self {
        Self {
            db,
            function_section: Default::default(),
            type_section: Default::default(),
            code_section: Default::default(),
            functions: Default::default(),
            codegen_queue: Default::default(),
        }
    }

    /// Generates all code reachable from the given fn instantiated with the given arguments.
    pub fn generate_from_fn(
        mut self,
        function: SymFunction<'db>,
        generics: Vec<ObjectGenericTerm<'db>>,
    ) -> wasm_encoder::Module {
        self.declare_fn(function, generics);
        while let Some(item) = self.codegen_queue.pop() {
            match item {
                CodegenQueueItem::Function(fn_key) => self.codegen_fn(fn_key),
            }
        }

        let mut module = wasm_encoder::Module::new();
        module.section(&self.type_section);
        module.section(&self.function_section);
        module.section(&self.code_section);

        module
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Update)]
pub(crate) struct FnKey<'db>(SymFunction<'db>, Vec<ObjectGenericTerm<'db>>);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Update)]
pub(crate) struct FnIndex(u32);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Update, FromImpls)]
enum CodegenQueueItem<'db> {
    Function(FnKey<'db>),
}
