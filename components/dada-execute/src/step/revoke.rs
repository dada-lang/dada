use dada_ir::{error, storage::Joint};

use crate::{
    error::DiagnosticBuilderExt,
    machine::{Permission, PermissionData, ValidPermissionData},
};

use super::Stepper;

impl Stepper<'_> {
    /// Revokes the given permission, recording the current PC as the "reason".
    #[tracing::instrument(level = "Debug", skip(self))]
    pub(super) fn revoke(&mut self, permission: Permission) -> eyre::Result<()> {
        let pc = self.machine.opt_pc();
        let p = std::mem::replace(&mut self.machine[permission], PermissionData::Expired(pc));

        if let PermissionData::Valid(ValidPermissionData {
            tenants, easements, ..
        }) = p
        {
            for easement in easements {
                self.revoke(easement)?;
            }

            for tenant in tenants {
                self.revoke(tenant)?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(level = "Debug", skip(self))]
    pub(super) fn forbid_tenants(&mut self, permission: Permission) -> eyre::Result<()> {
        // Report an error if `permission` has tenants.
        if let PermissionData::Valid(ValidPermissionData { tenants, .. }) =
            &self.machine[permission]
        {
            if let Some(&tenant) = tenants.first() {
                let tenant = self.machine[tenant].assert_valid();
                let span = self.machine.pc().span(self.db);
                return match tenant.joint {
                    Joint::No => Err(error!(span, "cannot write to leased data").eyre(self.db)),
                    Joint::Yes => Err(error!(span, "cannot write to shared data").eyre(self.db)),
                };
            }
        }

        Ok(())
    }

    /// Revoke any tenant of `permission` that is not currently
    /// sharing access to the object.
    ///
    /// Used when the object is read through `permission` (or a write
    /// to an atomic field).
    ///
    /// (There should be at most one such tenant.)
    #[tracing::instrument(level = "Debug", skip(self))]
    pub(super) fn forbid_exclusive_tenants(&mut self, permission: Permission) -> eyre::Result<()> {
        // Report an error if `permission` has tenants.
        if let PermissionData::Valid(ValidPermissionData { tenants, .. }) =
            &self.machine[permission]
        {
            for &tenant in tenants {
                let tenant = self.machine[tenant].assert_valid();
                if let Joint::No = tenant.joint {
                    let span = self.machine.pc().span(self.db);
                    return Err(error!(span, "cannot access leased data").eyre(self.db));
                }
            }
        }

        Ok(())
    }
}
