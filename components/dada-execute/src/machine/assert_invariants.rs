use dada_collections::Map;
use dada_ir::{
    code::bir,
    in_ir_db::InIrDbExt,
    storage::{Joint, Leased},
};
use salsa::DebugWithDb;

use crate::{
    ext::DadaExecuteClassExt,
    machine::{
        op::MachineOp, Frame, Object, ObjectData, Permission, PermissionData, Reservation, Value,
    },
};

use super::ReservationData;

pub(super) struct AssertInvariants<'me> {
    db: &'me dyn crate::Db,

    machine: &'me dyn MachineOp,

    /// Every permission ought to be associated with "at most one" object.
    permission_map: Map<Permission, Object>,
}

impl<'me> AssertInvariants<'me> {
    pub(super) fn new(db: &'me dyn crate::Db, machine: &'me dyn MachineOp) -> Self {
        Self {
            db,
            machine,
            permission_map: Default::default(),
        }
    }

    pub(super) fn assert_all_ok(&mut self) -> eyre::Result<()> {
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

            ObjectData::Reservation(r) => {
                let _object = self.assert_reservation_ok(*r)?;
            }

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

    /// Asserts that the reservation is ok and returns the reserved object.
    pub(super) fn assert_reservation_ok(
        &mut self,
        reservation: Reservation,
    ) -> eyre::Result<Object> {
        let ReservationData {
            pc: _,
            frame_index,
            place,
        } = self.machine[reservation];
        self.assert_reserved_place(reservation, &self.machine[frame_index], place)
    }

    /// Assert that the place `place` found in a reservation `reservation`
    /// is in fact reserved. We expect to find `reservation` in each permission
    /// and we expect this to be a unique place.
    fn assert_reserved_place(
        &mut self,
        reservation: Reservation,
        frame: &Frame,
        place: bir::Place,
    ) -> eyre::Result<Object> {
        let bir = frame.pc.bir;
        let table = &bir.data(self.db).tables;
        match &table[place] {
            bir::PlaceData::Class(_)
            | bir::PlaceData::Function(_)
            | bir::PlaceData::Intrinsic(_) => {
                eyre::bail!(
                    "reserved place `{:?}` bottoms out in a constant `{:?}`",
                    reservation,
                    table[place],
                );
            }

            bir::PlaceData::LocalVariable(lv) => {
                let value = frame.locals[*lv];
                self.assert_reserved_value(reservation, value)
            }

            bir::PlaceData::Dot(owner, field) => {
                let object = self.assert_reserved_place(reservation, frame, *owner)?;
                match &self.machine[object] {
                    ObjectData::Instance(instance) => {
                        let Some(index) = instance.class.field_index(self.db, *field) else {
                            eyre::bail!(
                                "reservation `{:?}` references place `{:?}` with invalid field `{:?}` for object `{:?}`",
                                reservation,
                                place.debug(&bir.in_ir_db(self.db)),
                                field.debug(self.db),
                                instance,
                            );
                            };
                        let value = instance.fields[index];
                        self.assert_reserved_value(reservation, value)
                    }

                    data => {
                        eyre::bail!(
                            "reservation `{:?}` reserved object with unexpected data `{:?}` at place `{:?}`",
                            reservation,
                            data,
                            place.debug(&bir.in_ir_db(self.db)),
                        );
                    }
                }
            }
        }
    }

    fn assert_reserved_value(
        &mut self,
        reservation: Reservation,
        value: Value,
    ) -> eyre::Result<Object> {
        let Value { object, permission } = value;

        let Some(valid) = self.machine[permission].valid() else {
            eyre::bail!(
                "reservation `{:?}` references expired permission `{:?}`",
                reservation,
                permission,
            );
        };

        if let Joint::Yes = valid.joint {
            eyre::bail!(
                "reservation `{:?}` references joint permission `{:?}`",
                reservation,
                permission,
            );
        }

        if !valid.reservations.contains(&reservation) {
            eyre::bail!(
                "reservation `{:?}` not found in reservation list `{:?}` for permission `{:?}`",
                reservation,
                valid.reservations,
                permission,
            );
        }

        Ok(object)
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
