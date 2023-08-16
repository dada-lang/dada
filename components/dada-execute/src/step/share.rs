use dada_ir::{
    code::bir,
    storage::{Joint, Leased},
};

use crate::machine::{ValidPermissionData, Value};

use super::{traversal::ObjectTraversal, Stepper};

impl Stepper<'_> {
    /// Shleasing an object creates a new permission that remains valid so long as the
    /// original reference is not used to do a write.
    ///
    /// # Examples
    ///
    /// Creates a shleased point:
    ///
    /// ```notrust
    /// p = Point(22, 44).shlease
    /// ```
    ///
    /// # Invariants
    ///
    /// The following invariants are maintained:
    ///
    /// * Results in a value `v` that refers to the same object as `place`
    /// * Always returns a shared value
    /// * `place` may be used for reads without invalidating the resulting value `v`
    /// * `place` remains valid and unchanged; writing to `place` may invalidate the result `v`.
    #[tracing::instrument(level = "Debug", skip(self, table))]
    pub(super) fn share_place(
        &mut self,
        table: &bir::Tables,
        place: bir::Place,
    ) -> eyre::Result<Value> {
        let object_traversal = self.traverse_to_object(table, place)?;
        self.shlease_traversal(object_traversal)
    }

    #[tracing::instrument(level = "Debug", skip(self))]
    pub(super) fn shlease_traversal(
        &mut self,
        object_traversal: ObjectTraversal,
    ) -> eyre::Result<Value> {
        // Shleasing something is akin to reading it.
        self.read(&object_traversal)?;

        let ObjectTraversal {
            object,
            accumulated_permissions,
        } = object_traversal;

        // The last traversed permission is the one that led to the object
        // (and there must be one, because you can't reach an object without
        // traversing at least one permission).
        let last_permission = *accumulated_permissions.traversed.last().unwrap();

        // Special case: If last permission is already shared, we can just duplicate it
        //
        // # Example
        //
        // ```notrust
        // a ----> [ Obj ]
        //   ===== [  f  ] --shared---> b
        //     |           ============ duplicate this permission
        //   can be any
        //   permission(s)
        // ```
        //
        // # Discussion
        //
        // Maintains our invariants:
        //
        // * Result is leased.
        // * Preserves sharing properties.
        // * `place` is not altered.
        if let ValidPermissionData {
            joint: Joint::Yes, ..
        } = self.machine[last_permission].assert_valid()
        {
            return Ok(Value {
                object,
                permission: last_permission,
            });
        }

        // If the input is `our`, clone it. Note that this preserves all the invariants,
        // even though it results in an owned value.
        if let (Joint::Yes, Leased::No) = (
            accumulated_permissions.joint,
            accumulated_permissions.leased,
        ) {
            let permission = self
                .machine
                .new_permission(ValidPermissionData::our(self.machine.pc()));
            return Ok(Value { object, permission });
        }

        // Otherwise, allocate a new joint lease permission.
        //
        // ## Examples
        //
        // In each case, we share `a.f`...
        //
        // ```notrust
        // a -my-> [ Obj ]
        //         [  f  ] --my--------> b
        //                   :
        //                   : tenant
        //                   v
        //                 --shleased--> b
        //                 =========== resulting permission
        // ```
        //
        // ```notrust
        // a -my-> [ Obj ]
        //         [  f  ] --leased---> b
        //                   :
        //                   : tenant
        //                   v
        //                 --shleased--> b
        //                 ============= resulting permission
        // ```
        //
        // ```notrust
        // a -our-> [ Obj ]
        //          [  f  ] --leased----> b
        //                    :
        //                    : tenant
        //                    v
        //                  --shleased--> b
        //                  ============= resulting permission
        // ```
        //
        // In each case, writing through `a.f` *may* invalidate the resulting
        // permission.
        //
        // # Discussion
        //
        // Maintains our invariants:
        //
        // * Result is shleased.
        // * Permissions for `place` are never altered.
        let permission = self.new_tenant_permission(Joint::Yes, &accumulated_permissions.traversed);

        Ok(Value { object, permission })
    }
}
