use dada_ir::{code::bir, storage_mode::Joint};

use crate::machine::{Object, ObjectData, Permission, PermissionData, Value};

use super::{traversal::ObjectTraversal, Stepper};

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
    pub(super) fn read(&mut self, traversal: &ObjectTraversal) -> Object {
        self.access(traversal, Self::revoke_exclusive_tenants);

        traversal.object
    }

    /// Read the object that was arrived at via the given traversal.
    /// This may cancel active leases along that path.
    /// Returns the object, which can now be accessed.
    ///
    /// Assumes that the traversal does not contain any expired
    /// permissions (creating a traversal fails if an expired
    /// permissions is encountered, so this could only happen
    /// if the traversal is "out of date" with respect to the machine
    /// state).
    pub(super) fn write(&mut self, traversal: &ObjectTraversal) {
        self.access(traversal, Self::revoke_tenants);
    }

    /// Helper for read/write:
    ///
    /// Apply `revoke_op` to each path that was traversed to reach the
    /// destination object `o`, along with any data exclusively
    /// reachable from `o`.
    fn access(
        &mut self,
        traversal: &ObjectTraversal,
        mut revoke_op: impl FnMut(&mut Self, bir::Place, Permission),
    ) {
        for &permission in &traversal.accumulated_permissions.traversed {
            assert!(matches!(self.machine[permission], PermissionData::Valid(_)));
            revoke_op(self, traversal.origin, permission);
        }

        self.for_each_reachable_exclusive_permission(traversal.origin, traversal.object, revoke_op);
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
        origin: bir::Place,
        object: Object,
        mut revoke_op: impl FnMut(&mut Self, bir::Place, Permission),
    ) {
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
                | ObjectData::Int(_)
                | ObjectData::String(_)
                | ObjectData::ThunkRust(_)
                | ObjectData::Unit(_)
                | ObjectData::Uint(_) => {}
            }
        }

        for p in reachable {
            revoke_op(self, origin, p);
        }
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
