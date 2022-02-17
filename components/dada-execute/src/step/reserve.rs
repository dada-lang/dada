use dada_ir::{code::bir, storage::Joint};

use crate::machine::{
    op::MachineOpExtMut, FrameIndex, ObjectData, PermissionData, ReservationData, Value,
};

use super::Stepper;

impl Stepper<'_> {
    /// The `reserve` operation
    pub(super) fn reserve_place(
        &mut self,
        table: &bir::Tables,
        place: bir::Place,
    ) -> eyre::Result<Value> {
        let object_traversal = self.traverse_to_object(table, place)?;

        // If the object is jointly accessible, then we can just share it and hold
        // on to that.
        if let Joint::Yes = object_traversal.accumulated_permissions.joint {
            return self.share_traversal(object_traversal);
        }

        // Otherwise, we have to reserve the place.

        let frame_index = FrameIndex::from(self.machine.frames().len() - 1);

        let reservation = self.machine.new_reservation(ReservationData {
            pc: self.machine.pc(),
            frame_index,
            place,
        });

        for &permission in &object_traversal.accumulated_permissions.traversed {
            let PermissionData::Valid(valid) = &mut self.machine[permission] else {
                panic!("traversed expired permision `{permission:?}`");
            };

            valid.reservations.push(reservation);
        }

        Ok(self.machine.my_value(ObjectData::Reservation(reservation)))
    }
}
