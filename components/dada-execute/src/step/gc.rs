//! The "garbage collector" looks for objects that have no owner
//! and collects them. The intent is that "compiled dada" would not
//! have a gc, but that it would be equivalent to the interpreter.
//!
//! The gc currently runs after every step, keeping things tidy.

use dada_collections::Set;
use dada_ir::storage_mode::Leased;

use crate::machine::{op::MachineOp, Object, ObjectData, Permission, PermissionData, Value};

use super::Stepper;

impl Stepper<'_> {
    /// Garbage collector: removes any objects that do not have an owner.
    /// Removes any permissions that do not appear in a live spot.
    ///
    /// Note: this relies on the Dada permission system for correctness.
    /// For example, if you have a lease on an object that is then
    /// freed, we assume that this lease would be revoked (and thus you would
    /// have an expired permission).
    #[tracing::instrument(level = "Debug", skip(self))]
    pub(super) fn gc(&mut self, in_flight_values: &[Value]) {
        let mut marks = Marks::default();
        Marker::new(self.machine, &mut marks).mark(in_flight_values);
        self.sweep(&marks);
    }
}

#[derive(Debug, Default)]
struct Marks {
    live_objects: Set<Object>,
    live_permissions: Set<Permission>,
}

struct Marker<'me> {
    machine: &'me dyn MachineOp,
    marks: &'me mut Marks,
}

impl<'me> Marker<'me> {
    fn new(machine: &'me dyn MachineOp, marks: &'me mut Marks) -> Self {
        Self { machine, marks }
    }

    #[tracing::instrument(level = "Debug", skip(self))]
    fn mark(&mut self, in_flight_values: &[Value]) {
        for frame in self.machine.frames() {
            for local_value in &frame.locals {
                self.mark_value(*local_value);
            }
        }

        for in_flight_value in in_flight_values {
            self.mark_value(*in_flight_value);
        }

        // the singleton unit object is always live :)
        self.marks.live_objects.insert(self.machine.unit_object());
    }

    #[tracing::instrument(level = "Debug", skip(self))]
    fn mark_values(&mut self, values: &[Value]) {
        for value in values {
            self.mark_value(*value);
        }
    }

    /// Marks a value that is reachable from something live (i.e., a root value).
    fn mark_value(&mut self, value: Value) {
        // The *permission* lives in a live spot, therefore it is live.
        // This also keeps "expired" permissions live.
        self.marks.live_permissions.insert(value.permission);

        let PermissionData::Valid(valid) = &self.machine[value.permission] else {
            tracing::debug!("marking expired permission but skipping object: {:?}", value);
            return;
        };

        if let Leased::Yes = valid.leased {
            // a lease alone isn't enough to keep data alive
            tracing::trace!("skipping leased value: {:?} valid={:?}", value, valid);
            return;
        }

        if !self.marks.live_objects.insert(value.object) {
            // already visited
            tracing::trace!("skipping already visited object: {:?}", value);
            return;
        }

        tracing::debug!("marking value: {:?}", value);
        let object_data: &ObjectData = &self.machine[value.object];
        match object_data {
            ObjectData::Instance(i) => self.mark_values(&i.fields),
            ObjectData::ThunkFn(f) => self.mark_values(&f.arguments),
            ObjectData::ThunkRust(f) => self.mark_values(&f.arguments),
            ObjectData::Tuple(t) => self.mark_values(&t.fields),

            ObjectData::Class(_)
            | ObjectData::Function(_)
            | ObjectData::Intrinsic(_)
            | ObjectData::Bool(_)
            | ObjectData::Uint(_)
            | ObjectData::Int(_)
            | ObjectData::Float(_)
            | ObjectData::String(_)
            | ObjectData::Unit(_) => {
                // no reachable data
            }
        }
    }
}

impl Stepper<'_> {
    #[tracing::instrument(level = "Debug", skip(self))]
    fn sweep(&mut self, marks: &Marks) {
        let mut live_permissions = self.machine.all_permissions();
        let mut dead_permissions = live_permissions.clone();
        live_permissions.retain(|p| marks.live_permissions.contains(p));
        dead_permissions.retain(|p| !marks.live_permissions.contains(p));

        // First: revoke all the dead permissions.
        for &p in &dead_permissions {
            tracing::debug!("revoking dead permission {:?}", p);
            self.revoke(p);
        }

        // Next: remove them from the heap.
        for &p in &dead_permissions {
            let data = self.machine.take_permission(p);
            tracing::debug!("removed dead permission {:?} = {:?}", p, data);
        }

        // Next: for each *live* permission, remove any dead tenants.
        for &p in &live_permissions {
            if let PermissionData::Valid(valid) = &mut self.machine[p] {
                valid.tenants.retain(|p| marks.live_permissions.contains(p));
            }
        }

        // Finally: remove dead objects.
        let mut dead_objects = self.machine.all_objects();
        dead_objects.retain(|o| !marks.live_objects.contains(o));

        for &o in &dead_objects {
            let data = self.machine.take_object(o);
            tracing::debug!("freeing {:?}: {:?}", o, data);
        }
    }
}
