use dada_ir::{code::bir, storage_mode::Joint};

use crate::machine::{Permission, PermissionData, ValidPermissionData};

use super::Stepper;

impl Stepper<'_> {
    pub(super) fn revoke(&mut self, origin: bir::Place, permission: Permission) {
        let p = std::mem::replace(
            &mut self.machine[permission],
            PermissionData::Expired(Some(origin)),
        );

        if let PermissionData::Valid(ValidPermissionData { tenants, .. }) = p {
            for tenant in tenants {
                self.revoke(origin, tenant);
            }
        }
    }

    pub(super) fn revoke_tenants(&mut self, origin: bir::Place, permission: Permission) {
        // Temporarily swap out the data for `permission`...
        let mut p = std::mem::replace(&mut self.machine[permission], PermissionData::Expired(None));

        // Cancel all the tenants and clear the list
        if let PermissionData::Valid(ValidPermissionData { tenants, .. }) = &mut p {
            for tenant in std::mem::take(tenants) {
                self.revoke(origin, tenant);
            }
        }

        // Put the (modified) data for `p` back
        self.machine[permission] = p;
    }

    pub(super) fn revoke_exclusive_tenants(&mut self, origin: bir::Place, permission: Permission) {
        // Temporarily swap out the data for `permission`...
        let mut p = std::mem::replace(&mut self.machine[permission], PermissionData::Expired(None));

        // Cancel all the exclusive tenants and remove them from the list
        if let PermissionData::Valid(ValidPermissionData { tenants, .. }) = &mut p {
            tenants.retain(|&tenant| match self.machine[tenant] {
                PermissionData::Valid(ValidPermissionData {
                    joint: Joint::No, ..
                }) => {
                    self.revoke(origin, tenant);
                    false
                }

                _ => true,
            });
        }

        // Put the (modified) data for `p` back
        self.machine[permission] = p;
    }
}
