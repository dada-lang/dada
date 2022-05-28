use dada_ir::intrinsic::Intrinsic;

use crate::{machine::Value, step::Stepper};

/// A "RustThunk" is a thunk implemented in Rust.
/// These are constructed from intrinsics.
/// The data interpreter doesn't a
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustThunk {
    pub(crate) description: &'static str,
    pub(crate) arguments: Vec<Value>,
    intrinsic: Intrinsic,
}

impl RustThunk {
    /// Creates a rust-based thunk -- when it is awaited, `invoke` will be called
    /// with the given values, and the resulting future awaited.
    pub(crate) fn new(
        description: &'static str,
        arguments: Vec<Value>,
        intrinsic: Intrinsic,
    ) -> Self {
        RustThunk {
            description,
            arguments,
            intrinsic,
        }
    }

    pub(crate) async fn invoke(self, stepper: &mut Stepper<'_>) -> eyre::Result<()> {
        let value = stepper
            .async_intrinsic(self.intrinsic, self.arguments)
            .await?;
        stepper.awaken(value)?;
        Ok(())
    }
}
