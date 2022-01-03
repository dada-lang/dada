use dada_ir::diagnostic::Fallible;

use crate::{interpreter::Interpreter, moment::Moment};

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

    pub(super) fn cancel(&self, interpreter: &Interpreter<'_>) -> Fallible<()> {
        self.canceled.check_still_valid(interpreter)?;
        Ok(self.tenant.cancel_tenant(interpreter))
    }

    pub(super) fn lease(&self, interpreter: &Interpreter<'_>) -> Fallible<Permission> {
        self.canceled.check_still_valid(interpreter)?;
        Ok(self.tenant.lease(interpreter))
    }

    pub(super) fn share(&self, interpreter: &Interpreter<'_>) -> Fallible<Permission> {
        self.canceled.check_still_valid(interpreter)?;
        Ok(self.tenant.share(interpreter))
    }

    pub(super) fn check_read(&self, interpreter: &Interpreter) -> Fallible<()> {
        self.canceled.check_still_valid(interpreter)?;
        Ok(self.tenant.cancel_tenant_if_exclusive(interpreter))
    }

    pub(super) fn check_write(&self, interpreter: &Interpreter) -> Fallible<()> {
        self.canceled.check_still_valid(interpreter)?;
        Ok(self.tenant.cancel_tenant(interpreter))
    }
}
