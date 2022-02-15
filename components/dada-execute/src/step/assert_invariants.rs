//! This code asserts various invariants that ought to hold across the machine state.
//! A failed assertion here represents a bug in dada's type system or operational semantics
//! and thus ought never to occur.
//!
//! Invariants:
//!
//! * I0: Every owned or exclusive permission should be associated with exactly one object across the entire machine.

use dada_collections::Map;
use dada_ir::storage_mode::{Joint, Leased};

use crate::machine::{op::MachineOp, Frame, Object, ObjectData, Permission, PermissionData, Value};

use super::Stepper;

impl Stepper<'_> {
    pub(crate) fn assert_invariants(&self) -> eyre::Result<()> {
        // Convert an assertion failure into a panic intentionally;
        // it's not the same as other sorts of failures.
        AssertInvariants::new(self.db, self.machine)
            .assert_all_ok()
            .unwrap();
        Ok(())
    }
}

struct AssertInvariants<'me> {
    machine: &'me dyn MachineOp,

    /// Every permission ought to be associated with "at most one" object.
    permission_map: Map<Permission, Object>,
}

impl<'me> AssertInvariants<'me> {
    fn new(_db: &'me dyn crate::Db, machine: &'me dyn MachineOp) -> Self {
        Self {
            machine,
            permission_map: Default::default(),
        }
    }

    fn assert_all_ok(&mut self) -> eyre::Result<()> {
        for frame in self.machine.frames() {
            self.assert_frame_ok(frame)?;
        }

        for object in self.machine.all_objects() {
            self.assert_object_ok(object)?;
        }

        for permission in self.machine.all_permissions() {
            self.assert_permission_ok(permission)?;
        }

        Ok(())
    }

    fn assert_frame_ok(&mut self, frame: &Frame) -> eyre::Result<()> {
        for v in &frame.locals {
            self.assert_value_ok(v)?;
        }

        Ok(())
    }

    fn assert_object_ok(&mut self, object: Object) -> eyre::Result<()> {
        let object_data: &ObjectData = &self.machine[object];
        match object_data {
            ObjectData::Instance(i) => self.assert_values_ok(&i.fields)?,
            ObjectData::ThunkFn(f) => self.assert_values_ok(&f.arguments)?,
            ObjectData::ThunkRust(f) => self.assert_values_ok(&f.arguments)?,
            ObjectData::Tuple(t) => self.assert_values_ok(&t.fields)?,

            ObjectData::Class(_)
            | ObjectData::Function(_)
            | ObjectData::Intrinsic(_)
            | ObjectData::Bool(_)
            | ObjectData::UnsignedInt(_)
            | ObjectData::Int(_)
            | ObjectData::SignedInt(_)
            | ObjectData::Float(_)
            | ObjectData::String(_)
            | ObjectData::Unit(_) => {
                // no reachable data
            }
        }
        Ok(())
    }

    fn assert_permission_ok(&mut self, _permission: Permission) -> eyre::Result<()> {
        Ok(())
    }

    fn assert_values_ok(&mut self, values: &[Value]) -> eyre::Result<()> {
        for v in values {
            self.assert_value_ok(v)?;
        }

        Ok(())
    }

    fn assert_value_ok(&mut self, value: &Value) -> eyre::Result<()> {
        let PermissionData::Valid(valid) = &self.machine[value.permission] else {
            return Ok(());
        };

        if let (_, Leased::No) | (Joint::No, _) = (valid.joint, valid.leased) {
            // Invariant I0: Every owned or exclusive permission should be associated with exactly one object across the entire machine.
            if let Some(other_object) = self.permission_map.insert(value.permission, value.object) {
                if value.object != other_object {
                    eyre::bail!(
                        "owned permission {:?} associated with at least two objects: {:?} and {:?}",
                        value.permission,
                        value.object,
                        other_object
                    );
                }
            }
        }

        Ok(())
    }
}
