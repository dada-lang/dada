use dada_ir::{
    error,
    storage::{Atomic, Joint, Leased},
};

use crate::{
    error::DiagnosticBuilderExt,
    machine::{Object, ObjectData, Permission, PermissionData, ValidPermissionData, Value},
};

use super::{
    traversal::{ObjectTraversal, PlaceTraversal},
    Stepper,
};

impl Stepper<'_> {
    /// Read the object that was arrived at via the given traversal.
    /// This may cancel active leases along that path.
    /// Returns the object, which can now be accessed.
    ///
    /// Assumes that the traversal does not contain any expired
    /// permissions (creating a traversal fails if an expired
    /// permissions is encountered, so this could only happen
    /// if the traversal is "out of date" with respect to the machine
    /// state).
    #[tracing::instrument(level = "Debug", skip(self))]
    pub(super) fn read(&mut self, traversal: &ObjectTraversal) -> eyre::Result<Object> {
        self.access(traversal, Self::forbid_exclusive_tenants)?;

        Ok(traversal.object)
    }

    /// Write to the object that was arrived at via the given traversal.
    /// This may cancel active leases along that path.
    ///
    /// Assumes that the traversal does not contain any expired
    /// permissions (creating a traversal fails if an expired
    /// permissions is encountered, so this could only happen
    /// if the traversal is "out of date" with respect to the machine
    /// state).
    #[tracing::instrument(level = "Debug", skip(self))]
    pub(super) fn write_object(&mut self, traversal: &ObjectTraversal) -> eyre::Result<()> {
        self.access(traversal, Self::forbid_tenants)
    }

    /// Given a traversal that has unique ownership, revokes the last permission
    /// in the path and returns the object. This also cancels tenants of traversed
    /// paths, as their (transitive) content has changed.
    #[tracing::instrument(level = "Debug", skip(self))]
    pub(super) fn take_object(&mut self, traversal: ObjectTraversal) -> eyre::Result<Object> {
        assert_eq!(traversal.accumulated_permissions.joint, Joint::No);
        assert_eq!(traversal.accumulated_permissions.leased, Leased::No);
        self.write_object(&traversal)?;
        let last_permission = *traversal.accumulated_permissions.traversed.last().unwrap();
        self.revoke(last_permission)?;
        Ok(traversal.object)
    }

    /// Write to the *place* identified by the given traversal (but not the
    /// object currently stored *in* that place). This may fail if the place
    /// is not writeable (e.g., if it is shared).
    ///
    /// Assumes that the traversal does not contain any expired
    /// permissions (creating a traversal fails if an expired
    /// permissions is encountered, so this could only happen
    /// if the traversal is "out of date" with respect to the machine
    /// state).
    #[tracing::instrument(level = "Debug", skip(self))]
    pub(super) fn write_place(&mut self, traversal: &PlaceTraversal) -> eyre::Result<()> {
        let ap = &traversal.accumulated_permissions;
        match (ap.joint, ap.atomic) {
            (Joint::Yes, Atomic::Yes) => {
                // Writing to a shared, atomic location NYI.
                //
                // We need to refactor traversal to track more information here,
                // since we need to distinguish between an atomic location in a
                // shared object vs a shared object in an (exclusive) atomic location.
                let span = self.machine.pc().span(self.db);
                return Err(error!(span, "atomic writes not implemented yet").eyre(self.db));
            }

            (Joint::Yes, Atomic::No) => {
                let span = self.machine.pc().span(self.db);
                return Err(error!(span, "cannot write to shared fields").eyre(self.db));
            }

            (Joint::No, Atomic::Yes) | (Joint::No, Atomic::No) => {
                // Writing to uniquely reachable data (whether or not it is atomic)
                // is legal, so long as the permissions along the way are not leased
                // or shared.
                //
                // # Example 1
                //
                // ```
                // p = a.lease
                // a.b = 3
                // /* use(p) */
                // ```
                //
                // This write to `a.b` fails with an error, because `a` is leased to `p`.
                //
                // # Example 2
                //
                // ```
                // p = a.b.lease
                // a.c = 3
                // /* use(p) */
                // ```
                //
                // This write to `a.c` does not fail: `p` has an easement on `a` but a lease on `a.b`,
                // so the write to `a.c` encounters no tenants.
                for &permission in &traversal.accumulated_permissions.traversed {
                    assert!(matches!(self.machine[permission], PermissionData::Valid(_)));
                    self.forbid_tenants(permission)?;
                }

                // # Prohibit writes to OWNERS of leased content
                //
                // The previous check forbids write to a leased *place* `P`.
                // But if P owns its value `v`, then writes to `P` will cause `v` to be freed.
                // If `v` is leased, that would invalidate the leasee.
                // Therefore, we forbid writes to a place `P` that owns a value `v`
                // which has tenants.
                //
                // Note that if `P` does not uniquely own its value, it is ok to overwrite it,
                // even if the value has tenants. This is because overwriting a leased value
                // does not free the original.
                //
                // # Example 1: Direct ownership
                //
                // ```
                // class C(f)
                // v = C(22)
                // p = v.lease
                // v = C(44)
                // ```
                //
                // Before the assignment to `a`, the object graph is as follows:
                //
                // ```
                // a --my------+---------> [ C ]
                //             |           [ f ] --our---> 22
                //             |
                // p --leased--+
                // ```
                //
                // We forbid the write because, if we permitted the write, the resulting
                // object graph would have a dangling reference:
                //
                // ```
                // a --my----------------> [ C ]
                //                         [ f ] --our---> 44
                //
                //             +---------> [ C ]
                //             |           [ f ] --our---> 22
                //             |
                // p --leased--+
                // ```
                //
                //
                // # Example 2: Indirect ownership
                //
                // ```
                // class C(f)
                // v = C(22)
                // a = C(v)
                // p = a.f.lease
                // a = C(44)
                // ```
                //
                // Before the assignment to `a`, the object graph is as follows:
                //
                // ```
                // a --my------> [ C ]
                //               [ f ] --my--+-> [ C ]
                //                           |   [ f ] --our--> 22
                //                           |
                // p --leased----------------+
                // ```
                //
                // We forbid the write because, if we permitted the write, the resulting
                // object graph would have a dangling reference:
                //
                // ```
                // a --my------> [ C ]
                //               [ f ] --our--> 44
                //
                //               [ C ]
                //               [ f ] --my--+-> [ C ]
                //                           |   [ f ] --our--> 22
                //                           |
                // p --leased----------------+
                // ```
                //
                // # Example 3: Lease
                //
                // ```
                // class C(f)
                // v = C(C(22))
                // a = C(v.lease)
                // p = a.f.f.lease
                // a.f = C(44)
                // ```
                //
                // Before the final assignment, the object graph is as follows:
                //
                // ```
                // v --my------------------------+--> [ C ]
                //                               |    [ f ] --my--+-> [ C ]
                //                               |                |   [ f ] --our--> 22
                // a --my------> [ C ]           |                |
                //               [ f ] --leased--+                |
                //                              | (easement)      |
                // p --leased-------------------+-----------------+
                // ```
                //
                // We permit the write. The resulting object graph is
                //
                // ```
                // v --my------------------------+--> [ C ]
                //                               |    [ f ] --my--+-> [ C ]
                //                               |                |   [ f ] --our--> 22
                //                               |                |
                //                         (lease/easement)       |
                //                               |                |
                // p --leased--------------------+-----------------+
                //
                // a --my--> [ C ]
                //           [ f ] --our--> 44
                // ```
                //
                // In particular, overwriting `a` did not free any values.
                // `p` retains its leasement on the lease of `v`, but that lease
                // is not attached to any paths anymore.
                let Value { permission, object } = self.peek(traversal.address);
                if let PermissionData::Valid(ValidPermissionData {
                    joint: Joint::No,
                    leased: Leased::No,
                    ..
                }) = self.machine[permission]
                {
                    self.for_each_reachable_exclusive_permission(object, Self::forbid_tenants)?;
                }
            }
        }

        Ok(())
    }

    /// Helper for read/write:
    ///
    /// Apply `revoke_op` to each path that was traversed to reach the
    /// destination object `o`, along with any data exclusively
    /// reachable from `o`.
    #[tracing::instrument(level = "Debug", skip(self, revoke_op))]
    fn access(
        &mut self,
        traversal: &ObjectTraversal,
        mut revoke_op: impl FnMut(&mut Self, Permission) -> eyre::Result<()>,
    ) -> eyre::Result<()> {
        for &permission in &traversal.accumulated_permissions.traversed {
            assert!(matches!(self.machine[permission], PermissionData::Valid(_)));
            revoke_op(self, permission)?;
        }

        self.for_each_reachable_exclusive_permission(traversal.object, revoke_op)?;

        Ok(())
    }

    /// Whenever an object is accessed (whether via a read or a write),
    /// that counts as an access to any content that is accessible
    /// via that object for the purposes of permission accounting.
    /// This function invokes `op` on each such permission.
    ///
    /// Note that merely *traversing* an object is not accessing it.
    /// Writing `print(a.b.c)` reads all the data from `c`, but
    /// only traverses `a.b`.
    ///
    /// # Examples
    ///
    /// This fragment of code
    ///
    /// ```notrust
    /// p = MyPair(a: MyPair(a: MyObject(), b: 44), c: 66)
    /// q = p.a.b.lease
    /// r = p.a.a.lease
    /// ```
    ///
    /// creates this object graph
    ///
    /// ```notrust
    /// p -my-> [ MyPair ]
    ///         [   a    ] --my--> [ MyPair ]
    ///         [        ]         [   a    ] --my--------> [ MyObject ]
    ///         [        ]         [   b    ] --our--> 44   ^
    ///         [        ]                              ^   |
    ///         [   b    ] --our-> 66                   |   |
    ///                                                 |   |
    ///                                                 |   |
    /// q -leased (joint)-------------------------------+   |
    /// r -leased (exclusive)-------------------------------+
    /// ```
    ///
    /// Now consider what happens if `s = p.a.share` is executed.
    /// The actual path `p.a` does not traverse any leases,
    /// but the data in `q` and `r` is now reachable from `s`
    /// (in addition to `p.a`).
    ///
    /// In the case of `q`, the reachable data (44) is via a joint path,
    /// so adding a new path (via `s`) doesn't make any difference.
    ///
    /// But in the case of `r`, the exclusive lease has to be canceled
    /// in order to make way for the new lease to `s`.
    fn for_each_reachable_exclusive_permission(
        &mut self,
        object: Object,
        mut revoke_op: impl FnMut(&mut Self, Permission) -> eyre::Result<()>,
    ) -> eyre::Result<()> {
        let mut reachable = vec![];
        let mut queue = vec![object];

        // Do a depth-first search and accumulate all exclusive permissions
        // reachable from `object` into `reachable`.
        //
        // We can't invoke `op` on them directly since we are holding a
        // reference onto `self.machine` to do the iteration:
        // since `op` can modify the machine state, this ensures that the
        // set of permissions we mutate will be a "snapshot" of how things
        // were at the time of the read (although I don't *believe* that
        // `op` can ever affect any of the permissions that we are traversing
        // here).
        while let Some(o) = queue.pop() {
            tracing::trace!("tracing(o = {:?})", o);
            match &self.machine[o] {
                ObjectData::Instance(instance) => {
                    self.push_reachable_via_fields(&instance.fields, &mut reachable, &mut queue);
                }

                ObjectData::ThunkFn(v) => {
                    self.push_reachable_via_fields(&v.arguments, &mut reachable, &mut queue);
                }

                ObjectData::Tuple(v) => {
                    self.push_reachable_via_fields(&v.fields, &mut reachable, &mut queue);
                }

                ObjectData::Bool(_)
                | ObjectData::Class(_)
                | ObjectData::Float(_)
                | ObjectData::Function(_)
                | ObjectData::Intrinsic(_)
                | ObjectData::SignedInt(_)
                | ObjectData::String(_)
                | ObjectData::ThunkRust(_)
                | ObjectData::Unit(_)
                | ObjectData::Int(_)
                | ObjectData::UnsignedInt(_) => {}
            }
        }

        for p in reachable {
            revoke_op(self, p)?;
        }

        Ok(())
    }

    fn push_reachable_via_fields(
        &self,
        fields: &[Value],
        reachable: &mut Vec<Permission>,
        queue: &mut Vec<Object>,
    ) {
        for value in fields {
            match &self.machine[value.permission] {
                PermissionData::Valid(v) => {
                    if let Joint::No = v.joint {
                        tracing::trace!("reaches {:?} with valid permission", value);
                        reachable.push(value.permission);
                        queue.push(value.object);
                    }
                }

                PermissionData::Expired(_) => {
                    // Is it an error to access an object that has a
                    // "hole"? I think not, we can wait to fault until
                    // you access that hole. Certainly we'd have to be
                    // careful around writes since (for example) one
                    // could be *patching* that hole! Have to think
                    // about this one.
                }
            }
        }
    }
}
