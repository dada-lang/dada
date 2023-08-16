use dada_ir::storage::{Joint, Leased};

use crate::machine::{Permission, ValidPermissionData};

use super::Stepper;

impl Stepper<'_> {
    /// Creates a new (joint or exclusive, depending on `joint`) permission;
    /// the permission will be a tenant of the last permission in `traversed`
    /// and will acquire easements on all the interim permissions.
    #[tracing::instrument(level = "debug", skip(self), ret)]
    pub(super) fn new_tenant_permission(
        &mut self,
        joint: Joint,
        traversed: &[Permission],
    ) -> Permission {
        let permission = self.machine.new_permission(ValidPermissionData {
            joint,
            leased: Leased::Yes,
            easements: vec![],
            tenants: vec![],
            pc: self.machine.pc(),
        });

        let (lessor, easements_on) = traversed.split_last().unwrap();

        // Make it a tenant of the last permission which was traversed.
        self.machine[*lessor]
            .assert_valid_mut()
            .tenants
            .push(permission);

        for easement_on in easements_on {
            self.machine[*easement_on]
                .assert_valid_mut()
                .easements
                .push(permission);
        }

        permission
    }
}
