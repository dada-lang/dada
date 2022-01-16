use dada_brew::prelude::BrewExt;
use dada_ir::function::Function;

use crate::{data::DadaFuture, execute::StackFrame, interpreter::Interpreter, value::Value};

pub(crate) struct Thunk {
    object: Box<dyn ThunkTrait>,
}

impl Thunk {
    pub(crate) fn new(
        closure: impl 'static
            + for<'i> FnOnce(&'i Interpreter<'_>, Option<&'i StackFrame<'_>>) -> DadaFuture<'i>,
    ) -> Self {
        Thunk {
            object: Box::new(closure),
        }
    }

    pub(crate) fn for_function(function: Function, arguments: Vec<Value>) -> Thunk {
        Thunk {
            object: Box::new(FunctionThunk {
                arguments,
                function,
            }),
        }
    }

    pub(crate) async fn invoke(
        self,
        interpreter: &Interpreter<'_>,
        parent_stack_frame: Option<&StackFrame<'_>>,
    ) -> eyre::Result<Value> {
        self.object.invoke(interpreter, parent_stack_frame).await
    }
}

impl std::fmt::Debug for Thunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Thunk").field(&"...").finish()
    }
}

#[async_trait::async_trait(?Send)]
trait ThunkTrait {
    async fn invoke(
        self: Box<Self>,
        interpreter: &Interpreter<'_>,
        parent_stack_frame: Option<&StackFrame<'_>>,
    ) -> eyre::Result<Value>;
}

#[async_trait::async_trait(?Send)]
impl<T> ThunkTrait for T
where
    T: for<'i> FnOnce(&'i Interpreter<'_>, Option<&'i StackFrame<'_>>) -> DadaFuture<'i>,
{
    async fn invoke(
        self: Box<Self>,
        interpreter: &Interpreter<'_>,
        parent_stack_frame: Option<&StackFrame<'_>>,
    ) -> eyre::Result<Value> {
        self(interpreter, parent_stack_frame).await
    }
}

struct FunctionThunk {
    arguments: Vec<Value>,
    function: Function,
}

#[async_trait::async_trait(?Send)]
impl ThunkTrait for FunctionThunk {
    async fn invoke(
        self: Box<Self>,
        interpreter: &Interpreter<'_>,
        parent_stack_frame: Option<&StackFrame<'_>>,
    ) -> eyre::Result<Value> {
        let bir = self.function.brew(interpreter.db());
        interpreter
            .execute_bir(bir, self.arguments, parent_stack_frame)
            .await
    }
}
