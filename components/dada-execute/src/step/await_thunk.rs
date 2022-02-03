use dada_brew::prelude::*;
use dada_ir::{
    code::bir,
    error,
    storage_mode::{Joint, Leased},
};

use crate::{
    error::DiagnosticBuilderExt,
    machine::{ObjectData, Permission, ThunkFn, ValidPermissionData},
    thunk::RustThunk,
};

use super::Stepper;

pub(super) enum AwaitResult {
    PushedNewFrame,
    RustThunk(RustThunk),
}

impl Stepper<'_> {
    pub(super) fn await_thunk(
        &mut self,
        table: &bir::Tables,
        thunk_place: bir::Place,
    ) -> eyre::Result<AwaitResult> {
        let thunk = self.give_place(table, thunk_place)?;

        self.check_await_permission(thunk_place, thunk.permission)?;
        assert!(
            self.machine[thunk.permission]
                .assert_valid()
                .tenants
                .is_empty(),
            "being given full ownership implies no tenants"
        );

        match self.machine.take_object(thunk.object) {
            ObjectData::ThunkFn(ThunkFn {
                function,
                arguments,
            }) => {
                let bir = function.brew(self.db);
                self.machine.push_frame(self.db, bir, arguments);
                Ok(AwaitResult::PushedNewFrame)
            }

            ObjectData::ThunkRust(rust_thunk) => Ok(AwaitResult::RustThunk(rust_thunk)),

            data => {
                let span = self.span_from_bir(thunk_place);
                Err(Self::unexpected_kind(self.db, span, &data, "a thunk"))
            }
        }
    }

    fn check_await_permission(
        &mut self,
        thunk_place: bir::Place,
        thunk_permission: Permission,
    ) -> eyre::Result<()> {
        let &ValidPermissionData { joint, leased, .. } =
            self.machine[thunk_permission].assert_valid();
        let primary_label = match (joint, leased) {
            (Joint::Yes, Leased::Yes) => "you only have a shared lease on the object",
            (Joint::No, Leased::Yes) => "you only have a lease on the object",
            (Joint::Yes, Leased::No) => "you only have shared access to the object",
            (Joint::No, Leased::No) => return Ok(()),
        };
        let span = self.span_from_bir(thunk_place);
        Err(error!(span, "awaiting something requires full ownership")
            .primary_label(primary_label)
            .eyre(self.db))
    }
}
