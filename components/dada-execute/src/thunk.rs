use std::{future::Future, pin::Pin};

use crate::{interpreter::Interpreter, value::Value};

///
pub(crate) trait Thunk: std::fmt::Debug + Send {
    fn invoke<'i>(
        self: Pin<Box<Self>>,
        interpreter: &'i Interpreter<'_>,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<Value>> + 'i>>;
}
