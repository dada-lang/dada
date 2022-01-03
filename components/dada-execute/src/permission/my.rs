use dada_ir::diagnostic::Fallible;

use crate::{interpreter::Interpreter, moment::Moment};

use super::{invalidated::Invalidated, tenant::Tenant, Permission, PermissionData};

#[derive(Debug)]
pub(super) struct My {
    granted: Moment,

    /// Owners permissions are invalidated when they are given
    /// away.
    given: Invalidated,

    tenant: Tenant,
}

impl From<My> for PermissionData {
    fn from(m: My) -> Self {
        Self::My(m)
    }
}

impl My {
    pub(super) fn new(interpreter: &Interpreter<'_>) -> Self {
        Self {
            granted: interpreter.moment_now(),
            given: Invalidated::default(),
            tenant: Tenant::default(),
        }
    }

    pub(super) fn give(&self, interpreter: &Interpreter<'_>) -> Fallible<Permission> {
        self.given.invalidate(interpreter)?;
        self.tenant.cancel_tenant(interpreter);
        let permission = Permission::my(interpreter);
        Ok(permission)
    }

    pub(super) fn lease(&self, interpreter: &Interpreter<'_>) -> Fallible<Permission> {
        self.given.check_still_valid(interpreter)?;
        Ok(self.tenant.lease(interpreter))
    }

    pub(super) fn share(&self, interpreter: &Interpreter<'_>) -> Fallible<Permission> {
        self.given.check_still_valid(interpreter)?;
        Ok(self.tenant.share(interpreter))
    }

    pub(super) fn check_read(&self, interpreter: &Interpreter) -> Fallible<()> {
        self.given.check_still_valid(interpreter)?;
        Ok(self.tenant.cancel_tenant_if_exclusive(interpreter))
    }

    pub(super) fn check_write(&self, interpreter: &Interpreter) -> Fallible<()> {
        self.given.check_still_valid(interpreter)?;
        Ok(self.tenant.cancel_tenant(interpreter))
    }
}
