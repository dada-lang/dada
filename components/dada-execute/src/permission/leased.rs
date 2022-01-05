use dada_ir::error;

use crate::{error::DiagnosticBuilderExt, interpreter::Interpreter, moment::Moment};

use super::{invalidated::Invalidated, tenant::Tenant, Permission, PermissionData};

/// Represents an "Exclusive Lease" (nobody else has access during the lease)
#[derive(Debug)]
pub(super) struct Leased {
    granted: Moment,

    /// Leased permissions are invalidated when they are canceled by
    /// their owner.
    canceled: Invalidated,
    tenant: Tenant,
}

impl From<Leased> for PermissionData {
    fn from(v: Leased) -> Self {
        Self::Leased(v)
    }
}

impl Leased {
    pub(super) fn new(interpreter: &Interpreter<'_>) -> Self {
        Self {
            granted: interpreter.moment_now(),
            canceled: Invalidated::default(),
            tenant: Tenant::default(),
        }
    }

    pub(super) fn cancel(&self, interpreter: &Interpreter<'_>) -> eyre::Result<()> {
        self.canceled.check_still_valid(interpreter)?;
        Ok(self.tenant.cancel_tenant(interpreter))
    }

    pub(super) fn lease(&self, interpreter: &Interpreter<'_>) -> eyre::Result<Permission> {
        self.canceled.check_still_valid(interpreter)?;
        Ok(self.tenant.lease(interpreter))
    }

    pub(super) fn share(&self, interpreter: &Interpreter<'_>) -> eyre::Result<Permission> {
        self.canceled.check_still_valid(interpreter)?;
        Ok(self.tenant.share(interpreter))
    }

    pub(super) fn check_read(&self, interpreter: &Interpreter) -> eyre::Result<()> {
        self.canceled.check_still_valid(interpreter)?;
        Ok(self.tenant.cancel_tenant_if_exclusive(interpreter))
    }

    pub(super) fn check_write(&self, interpreter: &Interpreter) -> eyre::Result<()> {
        self.canceled.check_still_valid(interpreter)?;
        Ok(self.tenant.cancel_tenant(interpreter))
    }

    pub(crate) fn check_await(&self, interpreter: &Interpreter) -> eyre::Result<()> {
        let span_now = interpreter.span_now();
        let span_then = interpreter.span(self.granted);
        Err(error!(span_now, "leased permission does not permit await")
            .secondary_label(span_then, "permission granted here")
            .eyre())
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.canceled.is_valid()
    }
}
