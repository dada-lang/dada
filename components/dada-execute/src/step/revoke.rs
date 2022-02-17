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
            tenants,
            reservations,
            ..
        }) = p
        {
            if let Some(reservation) = reservations.into_iter().next() {
                let data = &self.machine[reservation];
                let error_pc = pc.unwrap_or(data.pc);
                let error_span = error_pc.span(self.db);
                return Err(error!(
                    error_span,
                    "you can't overwrite this value, it is reserved right now"
                )
                .primary_label("attempting to invalidate reservation here")
                .secondary_label(data.pc.span(self.db), "reservation was placed here")
                .eyre(self.db));
            }

            for tenant in tenants {
                self.revoke(tenant)?;
            }
        }

        Ok(())
    }

    /// True if the permission `p` is currently sharing access to the object's
    /// fields. This is true if `permission` is a joint permission, but it's
    /// also true if it's a unique permission that is leased by a joint permission.
    fn is_sharing_access(&self, permission: Permission) -> bool {
        let Some(valid) = self.machine[permission].valid() else {
            return false;
        };

        if let Joint::Yes = valid.joint {
            return true;
        }

        valid.tenants.iter().any(|p| self.is_sharing_access(*p))
    }

    #[tracing::instrument(level = "Debug", skip(self))]
    pub(super) fn revoke_tenants(&mut self, permission: Permission) -> eyre::Result<()> {
        // Temporarily swap out the data for `permission`...
        let mut p = std::mem::replace(&mut self.machine[permission], PermissionData::Expired(None));

        // Cancel all the tenants and clear the list
        if let PermissionData::Valid(ValidPermissionData { tenants, .. }) = &mut p {
            for tenant in std::mem::take(tenants) {
                self.revoke(tenant)?;
            }
        }

        // Put the (modified) data for `p` back
        self.machine[permission] = p;

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
    pub(super) fn revoke_exclusive_tenants(&mut self, permission: Permission) -> eyre::Result<()> {
        // Temporarily swap out the data for `permission`...
        let mut p = std::mem::replace(&mut self.machine[permission], PermissionData::Expired(None));

        // Cancel all the exclusive tenants and remove them from the list
        if let PermissionData::Valid(ValidPermissionData { tenants, .. }) = &mut p {
            let mut result = Ok(());
            tenants.retain(|&tenant| {
                if result.is_err() {
                    true
                } else if !self.is_sharing_access(tenant) {
                    result = self.revoke(tenant);
                    false
                } else {
                    true
                }
            });
            result?;
        }

        // Put the (modified) data for `p` back
        self.machine[permission] = p;

        Ok(())
    }
}
