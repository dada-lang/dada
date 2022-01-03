use dada_ir::diagnostic::Fallible;

use crate::{interpreter::Interpreter, moment::Moment};

use super::{invalidated::Invalidated, tenant::Tenant, Permission, PermissionData};

/// Represents an "Exclusive Lease" (nobody else has access during the lease)
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
    pub(super) fn new(granted: Moment) -> Self {
        Self {
            granted,
            canceled: Invalidated::default(),
            tenant: Tenant::default(),
        }
    }

    pub(super) fn cancel(&self, interpreter: &Interpreter<'_>, moment: Moment) -> Fallible<()> {
        self.canceled.check_still_valid(interpreter, moment)?;
        self.tenant.cancel_tenant(interpreter, moment)?;
        Ok(())
    }

    pub(super) fn lease(
        &self,
        interpreter: &Interpreter<'_>,
        moment: Moment,
    ) -> Fallible<Permission> {
        self.canceled.check_still_valid(interpreter, moment)?;
        self.tenant.lease(interpreter, moment)
    }

    pub(super) fn share(
        &self,
        interpreter: &Interpreter<'_>,
        moment: Moment,
    ) -> Fallible<Permission> {
        self.canceled.check_still_valid(interpreter, moment)?;
        self.tenant.share(interpreter, moment)
    }
}
