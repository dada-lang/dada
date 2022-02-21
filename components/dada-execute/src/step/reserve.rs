use dada_ir::{code::bir, storage::Joint};

use crate::machine::{
    op::MachineOpExtMut, ObjectData, Permission, PermissionData, Reservation, ReservationData,
    Value,
};

use super::Stepper;

impl Stepper<'_> {
    /// The `reserve` operation
    #[tracing::instrument(level = "Debug", skip(self, table))]
    pub(super) fn reserve_place(
        &mut self,
        table: &bir::Tables,
        place: bir::Place,
    ) -> eyre::Result<Value> {
        let object_traversal = self.traverse_to_object(table, place)?;

        tracing::debug!(?object_traversal, "object_traversal");

        // If the object is jointly accessible, then we can just share it and hold
        // on to that.
        if let Joint::Yes = object_traversal.accumulated_permissions.joint {
            return self.share_traversal(object_traversal);
        }

        // Otherwise, we have to reserve the place.

        let frame_index = self.machine.top_frame_index().unwrap();

        let reservation = self.machine.new_reservation(ReservationData {
            pc: self.machine.pc(),
            frame_index,
            place,
        });

        for &permission in &object_traversal.accumulated_permissions.traversed {
            tracing::debug!("adding reservation {:?} to {:?}", reservation, permission);

            let PermissionData::Valid(valid) = &mut self.machine[permission] else {
                panic!("traversed expired permision `{permission:?}`");
            };

            valid.reservations.push(reservation);
        }

        Ok(self.machine.my_value(ObjectData::Reservation(reservation)))
    }

    /// Removes the given reservation from the given permisions.
    /// Panics if those permissions are not reserved with this reservation.
    #[tracing::instrument(level = "Debug", skip(self))]
    pub(super) fn remove_reservations(
        &mut self,
        reservation: Reservation,
        traversed: &[Permission],
    ) -> eyre::Result<()> {
        for &permission in traversed {
            let PermissionData::Valid(valid) = &mut self.machine[permission] else {
                panic!("traversed expired permision `{permission:?}`");
            };

            let Some(index) = valid.reservations.iter().position(|r| *r == reservation) else {
                panic!("reservation not found")
            };

            valid.reservations.remove(index);
        }
        Ok(())
    }
}
