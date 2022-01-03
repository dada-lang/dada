use arc_swap::ArcSwapOption;
use dada_ir::diagnostic::Fallible;

use crate::{interpreter::Interpreter, moment::Moment};

use super::{Permission, PermissionData};

/// Core struct for any unique permission
#[derive(Default)]
pub(super) struct Tenant {
    /// Has this permission been leased or shared?
    tenant: ArcSwapOption<PermissionData>,
}

impl Tenant {
    pub(super) fn lease(
        &self,
        interpreter: &Interpreter<'_>,
        moment: Moment,
    ) -> Fallible<Permission> {
        self.cancel_tenant(interpreter, moment)?;
        let perm = Permission::leased(moment);
        self.tenant.store(Some(perm.data.clone()));
        Ok(perm)
    }

    pub(super) fn share(
        &self,
        interpreter: &Interpreter<'_>,
        moment: Moment,
    ) -> Fallible<Permission> {
        self.cancel_tenant(interpreter, moment)?;
        let perm = Permission::shared(moment);
        self.tenant.store(Some(perm.data.clone()));
        Ok(perm)
    }

    pub(super) fn cancel_tenant(
        &self,
        interpreter: &Interpreter<'_>,
        moment: Moment,
    ) -> Fallible<()> {
        let tenant = self.tenant.load();
        if let Some(tenant) = &*tenant {
            tenant.cancel(interpreter, moment)?;
        }
        self.tenant.store(None);
        Ok(())
    }
}
