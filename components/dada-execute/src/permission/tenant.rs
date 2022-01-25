use arc_swap::ArcSwapOption;

use crate::interpreter::Interpreter;

use super::{Permission, PermissionData};

/// Core struct for any unique permission
#[derive(Default, Debug)]
pub(crate) struct Tenant {
    /// Has this permission been leased or shared?
    tenant: ArcSwapOption<PermissionData>,
}

impl Tenant {
    pub(super) fn lease(&self, interpreter: &Interpreter<'_>) -> Permission {
        self.cancel_tenant(interpreter);
        let perm = Permission::leased(interpreter);
        self.tenant.store(Some(perm.data.clone()));
        perm
    }

    pub(super) fn share(&self, interpreter: &Interpreter<'_>) -> Permission {
        self.cancel_tenant(interpreter);
        let perm = Permission::shared(interpreter);
        self.tenant.store(Some(perm.data.clone()));
        perm
    }

    pub(super) fn cancel_tenant(&self, interpreter: &Interpreter<'_>) {
        let tenant = self.tenant.load();
        if let Some(tenant) = &*tenant {
            tenant.cancel(interpreter).expect("failed to cancel tenant");
            self.tenant.store(None);
        }
    }

    pub(crate) fn cancel_tenant_if_exclusive(&self, interpreter: &Interpreter) {
        let tenant = self.tenant.load();
        if let Some(tenant) = &*tenant {
            if tenant.exclusive() {
                tenant.cancel(interpreter).expect("failed to cancel tenant");
                self.tenant.store(None);
            }
        }
    }

    pub(crate) fn peek_tenant(&self) -> Option<Permission> {
        self.tenant.load_full().map(Permission::new)
    }
}
