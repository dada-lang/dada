use dada_ir::error;

use crate::{error::DiagnosticBuilderExt, interpreter::Interpreter, moment::Moment};

use super::{invalidated::Invalidated, Permission, PermissionData};

#[derive(Debug)]
pub(super) struct Shared {
    granted: Moment,
    canceled: Invalidated,
}

impl From<Shared> for PermissionData {
    fn from(v: Shared) -> Self {
        Self::Shared(v)
    }
}

impl Shared {
    pub(super) fn new(interpreter: &Interpreter<'_, '_>) -> Self {
        Self {
            granted: interpreter.moment_now(),
            canceled: Default::default(),
        }
    }

    pub(super) fn cancel(&self, interpreter: &Interpreter<'_, '_>) -> eyre::Result<()> {
        self.canceled.invalidate(interpreter)?;
        Ok(())
    }

    pub(super) fn share(
        &self,
        this: &Permission,
        interpreter: &Interpreter<'_, '_>,
    ) -> eyre::Result<Permission> {
        self.canceled.check_still_valid(interpreter)?;
        Ok(this.duplicate())
    }

    pub(super) fn check_read(&self, interpreter: &Interpreter) -> eyre::Result<()> {
        self.canceled.check_still_valid(interpreter)
    }

    pub(super) fn check_write(&self, interpreter: &Interpreter) -> eyre::Result<()> {
        let span_now = interpreter.span_now();
        let span_then = interpreter.span(self.granted);
        Err(error!(span_now, "shared permission does not permit writes")
            .secondary_label(span_then, "permission granted here")
            .eyre(interpreter.db()))
    }

    pub(crate) fn check_await(&self, interpreter: &Interpreter) -> eyre::Result<()> {
        let span_now = interpreter.span_now();
        let span_then = interpreter.span(self.granted);
        Err(error!(span_now, "shared permission does not permit await")
            .secondary_label(span_then, "permission granted here")
            .eyre(interpreter.db()))
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.canceled.is_valid()
    }
}
