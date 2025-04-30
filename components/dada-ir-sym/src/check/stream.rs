use std::pin::Pin;

use dada_ir_ast::diagnostic::Errors;

use super::env::Env;

pub struct Consumer<'db, A, R> {
    op: Pin<Box<dyn ErasedConsumer<'db, A, R>>>,
}

impl<'db, A, R> Consumer<'db, A, R> {
    pub fn new(op: impl AsyncFnMut(&mut Env<'db>, A) -> R) -> Self {
        Consumer { op: Box::pin(op) }
    }

    pub async fn consume(&mut self, env: &mut Env<'db>, arg: A) -> R {
        self.op.consume(env, arg).await
    }
}

/// Dyn-safe wrapper around a closure.
trait ErasedConsumer<'db, A, R> {
    fn consume<'a>(
        &'a mut self,
        env: &'a mut Env<'db>,
        arg: A,
    ) -> Pin<Box<dyn Future<Output = R> + 'a>>;
}

impl<'db, F, A, R> ErasedConsumer<'db, A, R> for F
where
    F: AsyncFn(&mut Env<'db>, A) -> R,
{
    fn consume<'a>(
        &'a mut self,
        env: &'a mut Env<'db>,
        arg: A,
    ) -> Pin<Box<dyn Future<Output = R> + 'a>> {
        Box::pin(self(env, arg))
    }
}
