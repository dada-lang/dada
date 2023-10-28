use dada_ir::code::bir;

use crate::{
    machine::{stringify::DefaultStringify, ValidPermissionData},
    machine::{ObjectData, Value},
};

use super::Stepper;

impl Stepper<'_> {
    pub(super) fn concatenate(
        &mut self,
        table: &bir::Tables,
        places: &[bir::Place],
    ) -> eyre::Result<Value> {
        let mut string = String::new();
        for place in places {
            let value = self.share_place(table, *place)?;
            string.push_str(&self.machine.stringify_value(self.db, value));
        }

        Ok(Value {
            object: self.machine.new_object(ObjectData::String(string)),
            permission: self
                .machine
                .new_permission(ValidPermissionData::our(self.machine.pc())),
        })
    }
}
