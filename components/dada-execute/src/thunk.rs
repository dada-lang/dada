use crate::{machine::Value, step::Stepper};

/// A "RustThunk" is a thunk implemented in Rust.
/// These are constructed from intrinsics.
/// The data interpreter doesn't a
pub struct RustThunk {
    description: &'static str,
    arguments: Vec<Value>,
    object: Box<dyn RustThunkTrait>,
}

impl RustThunk {
    /// Creates a rust-based thunk -- when it is awaited, `invoke` will be called
    /// with the given values, and the resulting future awaited.
    pub(crate) fn new(
        description: &'static str,
        arguments: Vec<Value>,
        invoke: impl RustThunkTrait + 'static,
    ) -> Self {
        RustThunk {
            description,
            arguments,
            object: Box::new(invoke),
        }
    }

    pub(crate) async fn invoke(self, stepper: &mut Stepper<'_>) -> eyre::Result<()> {
        let value = self.object.invoke(stepper, self.arguments).await?;
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
pub(crate) trait RustThunkTrait {
    async fn invoke(
        self: Box<Self>,
        stepper: &mut Stepper<'_>,
        arguments: Vec<Value>,
    ) -> eyre::Result<Value>;
}
