use dada_ir::storage_mode::{Joint, Leased};

use crate::machine::{Permission, ValidPermissionData};

use super::Stepper;

impl Stepper<'_> {
    /// Creates a new (joint or exclusive, depending on `joint`) permission that is a tenant of `lessor`.
    pub(super) fn new_tenant_permission(&mut self, joint: Joint, lessor: Permission) -> Permission {
        let permission = self.machine.new_permission(ValidPermissionData {
            joint,
            leased: Leased::Yes,
            tenants: vec![],
        });

        // Make it a tenant of the last permission which was traversed.
        self.machine[lessor]
            .assert_valid_mut()
            .tenants
            .push(permission);

        permission
    }
}
