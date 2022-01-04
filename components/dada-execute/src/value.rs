use std::sync::Arc;

use dada_ir::{diagnostic::Fallible, func::Function, word::Word};
use parking_lot::Mutex;

use crate::{data::Data, interpreter::Interpreter, permission::Permission};

#[derive(Debug)]
pub(crate) struct Value {
    permission: Permission,
    data: Arc<Mutex<Data>>,
}

impl Value {
    pub(crate) fn new(interpreter: &Interpreter<'_>, value: impl Into<Data>) -> Value {
        Value {
            permission: Permission::my(interpreter),
            data: Arc::new(Mutex::new(value.into())),
        }
    }

    /// Give a closure read access to the data from this value.
    ///
    /// Can fail if the permission doesn't permit reads.
    pub(crate) fn read<R>(
        &self,
        interpreter: &Interpreter<'_>,
        op: impl FnOnce(&Data) -> Fallible<R>,
    ) -> Fallible<R> {
        self.permission.check_read(interpreter)?;
        op(&self.data.lock())
    }

    /// Give a closure write access to the data from this value.
    ///
    /// Can fail if the permission doesn't permit writes.
    pub(crate) fn write<R>(
        &self,
        interpreter: &Interpreter<'_>,
        op: impl FnOnce(&mut Data) -> Fallible<R>,
    ) -> Fallible<R> {
        self.permission.check_write(interpreter)?;
        op(&mut self.data.lock())
    }

    pub(crate) fn field<R>(
        &self,
        interpreter: &Interpreter<'_>,
        word: Word,
        op: impl FnOnce(&Value) -> Fallible<R>,
    ) -> Fallible<R> {
        self.permission.check_read(interpreter)?;
        op(self.data.lock().field(word)?)
    }

    pub(crate) fn give(&self, interpreter: &Interpreter<'_>) -> Fallible<Value> {
        Ok(Value {
            permission: self.permission.give(interpreter)?,
            data: self.data.clone(),
        })
    }

    pub(crate) fn into_share(self, interpreter: &Interpreter<'_>) -> Fallible<Value> {
        Ok(Value {
            permission: self.permission.into_share(interpreter)?,
            data: self.data.clone(),
        })
    }

    pub(crate) fn share(&self, interpreter: &Interpreter<'_>) -> Fallible<Value> {
        Ok(Value {
            permission: self.permission.share(interpreter)?,
            data: self.data.clone(),
        })
    }

    pub(crate) fn lease(&self, interpreter: &Interpreter<'_>) -> Fallible<Value> {
        Ok(Value {
            permission: self.permission.share(interpreter)?,
            data: self.data.clone(),
        })
    }
}
