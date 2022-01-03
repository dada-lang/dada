use dada_ir::diagnostic::Fallible;

use crate::{
    interpreter::{self, Interpreter},
    moment::Moment,
};

use super::{invalidated::Invalidated, tenant::Tenant, Permission, PermissionData};

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
    pub(super) fn new(granted: Moment) -> Self {
        Self {
            granted,
            given: Invalidated::default(),
            tenant: Tenant::default(),
        }
    }

    pub(super) fn give(
        &self,
        interpreter: &Interpreter<'_>,
        moment: Moment,
    ) -> Fallible<Permission> {
        self.given.invalidate(interpreter, moment)?;
        self.tenant.cancel_tenant(interpreter, moment)?;
        let permission = Permission::my(moment);
        Ok(permission)
    }

    pub(super) fn lease(
        &self,
        interpreter: &Interpreter<'_>,
        moment: Moment,
    ) -> Fallible<Permission> {
        self.given.check_still_valid(interpreter, moment)?;
        self.tenant.lease(interpreter, moment)
    }

    pub(super) fn share(
        &self,
        interpreter: &Interpreter<'_>,
        moment: Moment,
    ) -> Fallible<Permission> {
        self.given.check_still_valid(interpreter, moment)?;
        self.tenant.share(interpreter, moment)
    }
}
