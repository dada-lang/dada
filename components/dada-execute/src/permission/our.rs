use dada_ir::error;

use crate::{error::DiagnosticBuilderExt, interpreter::Interpreter, moment::Moment};

use super::{Permission, PermissionData};

#[derive(Debug)]
pub(crate) struct Our {
    granted: Moment,
}

impl From<Our> for PermissionData {
    fn from(v: Our) -> Self {
        Self::Our(v)
    }
}

impl Our {
    pub(super) fn new(interpreter: &Interpreter<'_>) -> Self {
        Self {
            granted: interpreter.moment_now(),
        }
    }

    pub(super) fn share(
        &self,
        this: &Permission,
        _interpreter: &Interpreter<'_>,
    ) -> eyre::Result<Permission> {
        Ok(this.duplicate())
    }

    pub(super) fn check_read(&self, _interpreter: &Interpreter) -> eyre::Result<()> {
        Ok(())
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
        true
    }
}
