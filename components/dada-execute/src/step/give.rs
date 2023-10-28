use dada_ir::{
    code::bir,
    storage::{Joint, Leased},
};

use crate::machine::{ValidPermissionData, Value};

use super::{traversal::ObjectTraversal, Stepper};

impl Stepper<'_> {
    /// # Invariants
    ///
    /// * The result preserves the ownership and sharing properties of the original:
    ///   * If input is owned, result is owned; if input is leased, result is leased.
    ///   * If input is exclusive, result is exclusive; if input is joint, resut is joint.
    /// * If input is shared, then giving does not disturb the original path but copies it
    /// * If input is leased, then giving does not disturb the original path but leases it
    /// * If returned value is fully owned, it will not have any active tenants
    ///
    /// Note that -- unlike sharing and leasing -- giving does NOT ensure that `place` remains
    /// valid afterwards! In particular, if you give something that you own, the only way to
    /// preserve both its ownership/sharing properties is to remove the original.
    #[tracing::instrument(level = "Debug", skip(self, table))]
    pub(super) fn give_place(
        &mut self,
        table: &bir::Tables,
        place: bir::Place,
    ) -> eyre::Result<Value> {
        let object_traversal = self.traverse_to_object(table, place)?;
        self.give_traversal(table, object_traversal)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub(super) fn give_traversal(
        &mut self,
        table: &bir::Tables,
        object_traversal: ObjectTraversal,
    ) -> eyre::Result<Value> {
        // Giving something that is jointly accessible is handled via sharing.
        //
        // e.g.
        //
        // ```notrust
        // p = Point(22, 44).share
        // q = p.give
        // ```
        //
        // # Discussion
        //
        // * Sharing preserves the ownership properties, just like give
        // * Sharing always results in a shared permission, but since input is shared, this also preserves desired properties
        if let Joint::Yes = object_traversal.accumulated_permissions.joint {
            return self.into_shared_traversal(object_traversal);
        }

        // Giving something that is leased is handled via leasing.
        //
        // e.g.
        //
        // ```notrust
        // p = Point(22, 44).lease
        // q = p.give
        // ```
        //
        // # Discussion
        //
        // * Leasing preserves the sharing properties, just like give
        // * Leasing always results in a leased permission, but since input is shared, this also preserves desired properties
        if let Leased::Yes = object_traversal.accumulated_permissions.leased {
            return self.lease_traversal(object_traversal);
        }

        // The value at `place` is exclusively owned: cancel the old permission (and any tenants)
        // create a new one to return.
        let object = self.take_object(object_traversal)?;

        let permission = self
            .machine
            .new_permission(ValidPermissionData::my(self.machine.pc()));
        Ok(Value { object, permission })
    }
}
