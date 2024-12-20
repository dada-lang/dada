use wasm_encoder::ValType;

use super::Cx;

pub(crate) struct FnTypeIndex(u32);

impl From<FnTypeIndex> for u32 {
    fn from(value: FnTypeIndex) -> Self {
        value.0
    }
}

impl<'db> Cx<'db> {
    /// Declares an instantiation of a function with a given set of arguments and returns its index.
    /// If the function is already declared, nothing happens.
    /// If the function is not already declared, it is enqueued for code-generation.
    pub(crate) fn declare_fn_type(
        &mut self,
        inputs: Vec<ValType>,
        outputs: Vec<ValType>,
    ) -> FnTypeIndex {
        let index = self.type_section.len();
        self.type_section.ty().function(inputs, outputs);
        FnTypeIndex(index)
    }
}
