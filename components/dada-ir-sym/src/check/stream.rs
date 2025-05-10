use std::pin::Pin;

use super::env::Env;

pub struct Consumer<'c, 'db, A, R>
where
    A: 'c,
    R: 'c,
{
    op: Box<dyn ErasedConsumer<'db, A, R> + 'c>,
}

impl<'c, 'db, A, R> Consumer<'c, 'db, A, R>
where
    A: 'c,
    R: 'c,
{
    pub fn new(op: impl AsyncFnMut(&mut Env<'db>, A) -> R + 'c) -> Self {
        Consumer { op: Box::new(op) }
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
    ) -> Pin<Box<dyn Future<Output = R> + 'a>>
    where
        A: 'a;
}

impl<'db, F, A, R> ErasedConsumer<'db, A, R> for F
where
    F: AsyncFnMut(&mut Env<'db>, A) -> R,
{
    fn consume<'a>(
        &'a mut self,
        env: &'a mut Env<'db>,
        arg: A,
    ) -> Pin<Box<dyn Future<Output = R> + 'a>>
    where
        A: 'a,
    {
        Box::pin(self(env, arg))
    }
}
