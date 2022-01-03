use std::sync::Arc;

use dada_ir::{diagnostic::Fallible, word::Word};

use crate::{data::Data, interpreter::Interpreter, permission::Permission};

#[derive(Debug)]
pub struct Value {
    permission: Permission,
    data: Arc<Data>,
}

impl Value {
    pub(super) fn field(&self, interpreter: &Interpreter<'_>, word: Word) -> Fallible<&Value> {
        self.permission.check_read(interpreter)?;
        self.data.field(word)
    }
}
