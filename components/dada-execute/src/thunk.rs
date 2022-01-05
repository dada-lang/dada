use crate::{data::DadaFuture, interpreter::Interpreter, value::Value};

pub(crate) type ThunkClosure = Box<dyn for<'i> FnOnce(&'i Interpreter<'_>) -> DadaFuture<'i>>;

pub(crate) struct Thunk {
    object: ThunkClosure,
}

impl Thunk {
    pub(crate) fn new(
        closure: impl 'static + for<'i> FnOnce(&'i Interpreter<'_>) -> DadaFuture<'i>,
    ) -> Self {
        Thunk {
            object: Box::new(closure),
        }
    }

    pub(crate) async fn invoke(self, interpreter: &Interpreter<'_>) -> eyre::Result<Value> {
        (self.object)(interpreter).await
    }
}

impl std::fmt::Debug for Thunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Thunk").field(&"...").finish()
    }
}
