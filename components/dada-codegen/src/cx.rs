use dada_ir_sym::{ir::functions::SymFunction, ir::types::SymGenericTerm};
use dada_util::{FromImpls, Map};
use salsa::Update;
use wasm_encoder::{CodeSection, ExportKind, ExportSection, FunctionSection, TypeSection};

mod generate_expr;
mod generate_fn;
mod wasm_fn_type;
mod wasm_repr;

/// Core codegen context.
pub(crate) struct Cx<'db> {
    db: &'db dyn crate::Db,
    function_section: FunctionSection,
    type_section: TypeSection,
    code_section: CodeSection,
    export_section: ExportSection,
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
            export_section: Default::default(),
            functions: Default::default(),
            codegen_queue: Default::default(),
        }
    }

    /// Generates all code reachable from the given fn instantiated with the given arguments.
    pub fn generate_from_fn(
        mut self,
        function: SymFunction<'db>,
        generics: Vec<SymGenericTerm<'db>>,
    ) -> wasm_encoder::Module {
        let index = self.declare_fn(function, generics);
        while let Some(item) = self.codegen_queue.pop() {
            match item {
                CodegenQueueItem::Function(fn_key) => self.codegen_fn(fn_key),
            }
        }

        let name = function.name(self.db).text(self.db);
        self.export_section.export(name, ExportKind::Func, index.0);

        let mut module = wasm_encoder::Module::new();
        module.section(&self.type_section);
        module.section(&self.function_section);
        module.section(&self.export_section);
        module.section(&self.code_section);

        module
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Update)]
pub(crate) struct FnKey<'db>(SymFunction<'db>, Vec<SymGenericTerm<'db>>);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Update)]
pub(crate) struct FnIndex(u32);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Update, FromImpls)]
enum CodegenQueueItem<'db> {
    Function(FnKey<'db>),
}
