use dada_ir::{diagnostic::Fallible, error};

use crate::{interpreter::Interpreter, moment::Moment};

use super::{Permission, PermissionData};

#[derive(Debug)]
pub(super) struct Our {
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
        interpreter: &Interpreter<'_>,
    ) -> Fallible<Permission> {
        Ok(this.duplicate())
    }

    pub(super) fn check_read(&self, interpreter: &Interpreter) -> Fallible<()> {
        Ok(())
    }

    pub(super) fn check_write(&self, interpreter: &Interpreter) -> Fallible<()> {
        let span_now = interpreter.span_now();
        let span_then = interpreter.span(self.granted);
        Err(error!(span_now, "shared permission does not permit writes")
            .secondary_label(span_then, "permission granted here")
            .emit(interpreter.db()))
    }
}
