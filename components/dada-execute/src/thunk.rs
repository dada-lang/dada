use crate::{data::DadaFuture, interpreter::Interpreter, value::Value};

pub(crate) struct Thunk {
    object: Box<dyn ThunkTrait>,
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
        self.object.invoke(interpreter).await
    }
}

impl std::fmt::Debug for Thunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Thunk").field(&"...").finish()
    }
}

#[async_trait::async_trait(?Send)]
trait ThunkTrait {
    async fn invoke(self: Box<Self>, interpreter: &Interpreter<'_>) -> eyre::Result<Value>;
}

#[async_trait::async_trait(?Send)]
impl<T> ThunkTrait for T
where
    T: for<'i> FnOnce(&'i Interpreter<'_>) -> DadaFuture<'i>,
{
    async fn invoke(self: Box<Self>, interpreter: &Interpreter<'_>) -> eyre::Result<Value> {
        self(interpreter).await
    }
}
