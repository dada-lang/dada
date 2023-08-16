use dada_ir::{
    code::bir,
    storage::{Joint, Leased},
};

use crate::machine::{ValidPermissionData, Value};

use super::{traversal::ObjectTraversal, Stepper};

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
    ///   * If original path was leased, shared result is leased and lives as long as original lease would
    ///     (sharing a `leased(p) Foo` gives a `shared(p) Foo`).
    /// * After sharing, the original path can be read (or shared again) without disturbing the share.
    ///
    /// Implication:
    ///
    /// * Sharing a shared thing is effectively "cloning" it, in the Rust sense
    #[tracing::instrument(level = "Debug", skip(self, table))]
    pub(super) fn into_shared_place(
        &mut self,
        table: &bir::Tables,
        place: bir::Place,
    ) -> eyre::Result<Value> {
        let object_traversal = self.traverse_to_object(table, place)?;
        self.into_shared_traversal(object_traversal)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub(super) fn into_shared_traversal(
        &mut self,
        traversal: ObjectTraversal,
    ) -> eyre::Result<Value> {
        // Sharing counts as a read of the data being shared.
        self.read(&traversal)?;

        // The last traversed permission is the one that led to the object
        // (and there must be one, because you can't reach an object without
        // traversing at least one permission).
        let last_permission = *traversal.accumulated_permissions.traversed.last().unwrap();

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
                object: traversal.object,
                permission: last_permission,
            });
        }

        match (
            traversal.accumulated_permissions.leased,
            traversal.accumulated_permissions.joint,
        ) {
            // If this path has (exclusive) ownership, we revoke
            // that permission to create shared ownership.
            //
            // We have to revoke the existing permission because the path
            // may be typed with a `my` type, and that cannot co-exist with
            // the object having a shared permission..
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
            // ...revokes the old permission and creates a new one:
            //
            // ```notrust
            // a -my-> [ Obj ]
            //         [  f  ] --X         b
            //                 ===         ^
            //                 revoked     |
            //                             |
            //              [] --our-------+
            //                 =============
            //                 ...and this permission
            //                 is created
            // ```
            (Leased::No, Joint::No) => {
                let object = self.take_object(traversal)?;
                let permission = self
                    .machine
                    .new_permission(ValidPermissionData::our(self.machine.pc()));
                Ok(Value { object, permission })
            }

            // If object is owned + shared, then we can just create a fresh
            // our permission. Note that, because we detected the special case
            // where the last step was a "shared" permission above, and because
            // we know the object is not leased, the `last_permission` here must
            // be a `my` permission, as in this example:
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
            //          [  f  ] --my------> b
            //                              ^
            //                              |
            //                              |
            //               [] --our-------+
            // ```
            //
            // Justification:
            //
            // * Object is already in a shared ownership state, so we can duplicate those
            //   permissions at will.
            //
            // Implications:
            //
            // * The only way to limit sharing is with leasing: even if you dynamically test
            //   the ref count of `a`, you cannot "unwrap" it and return to "sole ownership"
            //   state, because there may be extant references to the data that it owned.
            // * `our [String]` is therefore very different from `Rc<Vec<String>>`.
            (Leased::No, Joint::Yes) => {
                let permission = self
                    .machine
                    .new_permission(ValidPermissionData::our(self.machine.pc()));
                Ok(Value {
                    object: traversal.object,
                    permission,
                })
            }

            // We don't own the object, so create a joint leased permission.
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
            // ## Why not change the final lease to shared?
            //
            // When sharing something that is leased, we create a sublease rather
            // rather converting the lease itself to be a shared lease. The answer
            // is that this final lease may not be under our control.
            //
            // Consider this example:
            //
            // ```notrust
            // a -leased-> [ Obj ]
            //             [  f  ] --leased--> b
            // ```
            //
            // Here, there are two leases at play. `a` is itself leased, and it
            // contains a leased reference to `b`. This implies that *somewhere else*,
            // there is an *owner* for `a`. Let's call them `o`. So `o` owns an
            // object which has a leased value to `b`, and they've leased it out to `a`.
            // They expect to be able to re-assert control over that object at any
            // time and find it in the same state in which they left it.
            // If we permitted `a` to convert the lease to `b` into a shared lease,
            // that would violate `o`'s expectations.
            //
            // In other words, we want the owner of `o`  to be able to do this:
            //
            //     a = o.lease
            //     arbitraryCode(a)
            //     o.f.field += 1      # requires leased access
            //
            // and have it work, without concern for what `arbitraryCode` may do.
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
            (Leased::Yes, _) => {
                let permission = self.new_tenant_permission(
                    Joint::Yes,
                    &traversal.accumulated_permissions.traversed,
                );
                Ok(Value {
                    object: traversal.object,
                    permission,
                })
            }
        }
    }
}
