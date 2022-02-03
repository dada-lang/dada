use std::{future::Future, pin::Pin};

use crate::{machine::Value, step::Stepper};

/// A "RustThunk" is a thunk implemented in Rust.
/// These are constructed from intrinsics.
/// The data interpreter doesn't a
pub struct RustThunk {
    description: &'static str,
    object: Box<dyn RustThunkTrait>,
}

pub type RustFuture<'i> = Pin<Box<dyn Future<Output = eyre::Result<Value>> + 'i>>;

impl RustThunk {
    pub(crate) fn new(
        description: &'static str,
        closure: impl 'static + for<'i> FnOnce(&'i mut Stepper<'_>) -> RustFuture<'i>,
    ) -> Self {
        RustThunk {
            description,
            object: Box::new(closure),
        }
    }

    pub(crate) async fn invoke(self, stepper: &mut Stepper<'_>) -> eyre::Result<()> {
        let value = self.object.invoke(stepper).await?;
        stepper.awaken(value)?;
        Ok(())
    }
}

impl std::fmt::Debug for RustThunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(self.description).field(&"...").finish()
    }
}

#[async_trait::async_trait(?Send)]
trait RustThunkTrait {
    async fn invoke(self: Box<Self>, stepper: &mut Stepper<'_>) -> eyre::Result<Value>;
}

#[async_trait::async_trait(?Send)]
impl<T> RustThunkTrait for T
where
    T: for<'i> FnOnce(&'i mut Stepper<'_>) -> RustFuture<'i>,
{
    async fn invoke(self: Box<Self>, stepper: &mut Stepper<'_>) -> eyre::Result<Value> {
        self(stepper).await
    }
}
