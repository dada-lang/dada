use dada_ir::{
    code::bir,
    storage_mode::{Joint, Leased},
};

use crate::machine::Value;

use super::{
    traversal::{Anchor, ObjectTraversal},
    Stepper,
};

impl Stepper<'_> {
    /// The `share` operation converts a permission into a shared permission,
    /// if it is not already, and then returns a second value with that
    /// same shared permission.
    ///
    /// # Examples
    ///
    /// Creates a shared point:
    ///
    /// ```notrust
    /// p = Point(22, 44).share
    /// ```
    ///
    /// # Invariants
    ///
    /// * The result is shared
    /// * The shared result has the same ownership/lease properties as original path:
    ///   * If original path was owned, shared result is owned (sharing a `my Foo` gives an `our Foo`).
    ///   * If original path was leased, shared result is leased and lives as long as original lease would (sharing a `my leased(p) Foo` gives an `our leased(p) Foo`).
    /// * After sharing, the original path can be read (or shared again) without disturbing the share.
    ///
    /// Implication:
    ///
    /// * Sharing a shared thing is effectively "cloning" it, in the Rust sense
    pub(super) fn share_place(
        &mut self,
        table: &bir::Tables,
        place: bir::Place,
    ) -> eyre::Result<Value> {
        let anchor = Anchor::default();
        let object_traversal = self.traverse_to_object(&anchor, table, place)?;
        self.share_traversal(object_traversal)
    }

    pub(super) fn share_traversal(&mut self, traversal: ObjectTraversal) -> eyre::Result<Value> {
        // Sharing counts as a read of the data being shared.
        self.read(&traversal);

        let ObjectTraversal {
            origin: _,
            object,
            accumulated_permissions,
        } = traversal;

        // The last traversed permission is the one that led to the object
        // (and there must be one, because you can't reach an object without
        // traversing at least one permision).
        let last_permission = *accumulated_permissions.traversed.last().unwrap();

        // Special case, for simplicity and efficiency: If the final permission to the object
        // is joint, we can just duplicate it. The resulting permission
        //
        // # Examples
        //
        // Sharing `a.f` in these scenarios duplicates the existing permission to `b`:
        //
        // ```notrust
        // a -my-> [ Obj ]
        //         [  f  ] --shared--> b
        //                 =========== duplicated
        //
        // a -my-> [ Obj ]
        //         [  f  ] --our-----> b
        //                 =========== duplicated
        // ```
        if let Joint::Yes = self.machine[last_permission].assert_valid().joint {
            return Ok(Value {
                object,
                permission: last_permission,
            });
        }

        // Otherwise, if this path owns the object, we can convert that last permission
        // to be joint.
        //
        // # Examples
        //
        // Sharing `a.f` in this scenario...
        //
        // ```notrust
        // a -my-> [ Obj ]
        //         [  f  ] --my------> b
        // ```
        //
        // ...modifies the existing permission to `b`
        // and then duplicates it, yielding (respectively)...
        //
        // ```notrust
        // a -my-> [ Obj ]
        //         [  f  ] --our-----> b
        //                   ===       ^
        //                converted... |
        //                             |
        //              [] --our-------+
        //                 ============= ...then duplicated
        //                               to create this
        // ```
        //
        // Another example:
        //
        // ```notrust
        // a -our-> [ Obj ]
        //          [  f  ] --my------> b
        // ```
        //
        // becomes
        //
        // ```notrust
        // a -our-> [ Obj ]
        //          [  f  ] --our-----> b
        //                    ===       ^
        //                 converted... |
        //                              |
        //               [] --our-------+
        //                 ============= ...then duplicated
        //                               to create this
        // ```
        //
        // # Justification and discussion
        //
        // This preserves the invariants:
        //
        // * Result is shared
        // * Result is owned, just like input
        // * Original path can be read without disturbing input
        //
        // It mildly surprised me at first to convert the original path from `my` to `our`, but it's the only way to preserve the invariants:
        //
        // * If we were to sublease, result would not be owned, so when original was dropped, result would become invalid. But we promised
        //   result is owned iff input is owned.
        // * If we were to revoke the `my` permission, you would no longer be able to read the original path, but we promised original path remains valid.
        // * If we *just* created a new permission, the `my` permission would be invalid, but we promised original path remains valid.
        //
        // It's also what you want for something like `var p = Point(22, 44).share`.
        if let Leased::No = accumulated_permissions.leased {
            self.machine[last_permission].assert_valid_mut().joint = Joint::Yes;

            return Ok(Value {
                object,
                permission: last_permission,
            });
        }

        // Otherwise, we don't own the object, so create a joint leased permision.
        // This will remain valid so long as the lease is valid (and not re-asserted).
        //
        // # Examples
        //
        // Sharing `a.f` in this scenarios...
        //
        // ```notrust
        // a -leased-> [ Obj ]
        //             [  f  ] --my------> b
        // ```
        //
        // ...creates a shared sublease of the `my` permission:
        //
        // ```notrust
        // a -leased-> [ Obj ]
        //             [  f  ] --my------> b
        //                       :         ^
        //                       : tenant  |
        //                       v         |
        //                  [] --shared----+
        // ```
        //
        // Another example:
        //
        // ```notrust
        // a -my-> [ Obj ]
        //         [  f  ] --leased--> b
        // ```
        //
        // becomes
        //
        // ```notrust
        // a -my-> [ Obj ]
        //         [  f  ] --leased--> b
        //                   :         ^
        //                   : tenant  |
        //                   v         |
        //              [] --shared----+
        // ```
        //
        // ## Note
        //
        // Because of the special case for a "last permission", this case
        // is handled differently:
        //
        // ```notrust
        // a -leased-> [ Obj ]
        //             [  f  ] --shared----> b
        // ```
        //
        // Instead of creating a tenant of the final shared permission,
        // we simply clone it. But creating a tenant would be "ok" too,
        // just less efficient and maybe a bit confusing.
        let permission = self.new_tenant_permission(Joint::Yes, last_permission);

        Ok(Value { object, permission })
    }
}
