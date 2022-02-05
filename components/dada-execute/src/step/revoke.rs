use dada_ir::storage_mode::Joint;

use crate::machine::{Permission, PermissionData, ValidPermissionData};

use super::Stepper;

impl Stepper<'_> {
    /// Revokes the given permission, recording the current PC as the "reason".
    #[tracing::instrument(level = "Debug", skip(self))]
    pub(super) fn revoke(&mut self, permission: Permission) {
        let pc = self.machine.opt_pc();
        let p = std::mem::replace(&mut self.machine[permission], PermissionData::Expired(pc));

        if let PermissionData::Valid(ValidPermissionData { tenants, .. }) = p {
            for tenant in tenants {
                self.revoke(tenant);
            }
        }
    }

    #[tracing::instrument(level = "Debug", skip(self))]
    pub(super) fn revoke_tenants(&mut self, permission: Permission) {
        // Temporarily swap out the data for `permission`...
        let mut p = std::mem::replace(&mut self.machine[permission], PermissionData::Expired(None));

        // Cancel all the tenants and clear the list
        if let PermissionData::Valid(ValidPermissionData { tenants, .. }) = &mut p {
            for tenant in std::mem::take(tenants) {
                self.revoke(tenant);
            }
        }

        // Put the (modified) data for `p` back
        self.machine[permission] = p;
    }

    #[tracing::instrument(level = "Debug", skip(self))]
    pub(super) fn revoke_exclusive_tenants(&mut self, permission: Permission) {
        // Temporarily swap out the data for `permission`...
        let mut p = std::mem::replace(&mut self.machine[permission], PermissionData::Expired(None));

        // Cancel all the exclusive tenants and remove them from the list
        if let PermissionData::Valid(ValidPermissionData { tenants, .. }) = &mut p {
            tenants.retain(|&tenant| match self.machine[tenant] {
                PermissionData::Valid(ValidPermissionData {
                    joint: Joint::No, ..
                }) => {
                    self.revoke(tenant);
                    false
                }

                _ => true,
            });
        }

        // Put the (modified) data for `p` back
        self.machine[permission] = p;
    }
}
