use dada_brew::prelude::*;
use dada_ir::{code::bir, error, signature::InputTy, word::Word};
use dada_validate::prelude::*;

use crate::{
    error::DiagnosticBuilderExt,
    machine::{op::MachineOpExtMut, Instance, ObjectData, ProgramCounter, ThunkFn, Value},
    step::intrinsic::IntrinsicDefinition,
};

use super::Stepper;

pub(super) enum CallResult {
    Returned(Value),
    PushedNewFrame,
}

impl Stepper<'_> {
    pub(super) fn call(
        &mut self,
        table: &bir::Tables,
        pc: ProgramCounter,
        callee: bir::Place,
        argument_places: &[bir::Place],
        labels: &[Option<bir::Name>],
    ) -> eyre::Result<CallResult> {
        let function_value = self.give_place(table, callee)?;

        assert!(
            self.machine[function_value.permission].valid().is_some(),
            "giving place yielded value with invalid permissions"
        );

        match &self.machine[function_value.object] {
            &ObjectData::Class(c) => {
                let signature = c.signature(self.db);
                self.match_labels(table, pc, labels, &signature.inputs)?;
                let arguments = self.give_arguments(table, argument_places)?;
                self.check_signature(&arguments, signature)?;
                let instance = Instance {
                    class: c,
                    fields: arguments,
                };
                Ok(CallResult::Returned(
                    self.machine.my_value(self.machine.pc(), instance),
                ))
            }
            &ObjectData::Function(function) => {
                let signature = function.signature(self.db);
                self.match_labels(table, pc, labels, &signature.inputs)?;

                let arguments = self.give_arguments(table, argument_places)?;

                let expected_return_ty = self.check_signature(&arguments, signature)?;

                if function.effect(self.db).permits_await() {
                    // If the function can await, then it must be an async function.
                    // Now that we have validated the arguments, return a thunk.
                    let thunk = self.machine.my_value(
                        self.machine.pc(),
                        ThunkFn {
                            function,
                            arguments,
                            expected_return_ty,
                        },
                    );
                    Ok(CallResult::Returned(thunk))
                } else {
                    // This is not an async function, so push it onto the stack
                    // and begin execution immediately.
                    let bir = function.brew(self.db);
                    self.machine
                        .push_frame(self.db, bir, arguments, expected_return_ty);
                    Ok(CallResult::PushedNewFrame)
                }
            }
            &ObjectData::Intrinsic(intrinsic) => {
                let definition = IntrinsicDefinition::for_intrinsic(self.db, intrinsic);
                self.match_labels(table, pc, labels, &definition.argument_names)?;
                let arguments = self.give_arguments(table, argument_places)?;
                let value = (definition.function)(self, arguments)?;
                Ok(CallResult::Returned(value))
            }
            data => {
                let span = self.span_from_bir(callee);
                Err(error!(
                    span,
                    "expected something callable, found {}",
                    data.kind_str(self.db)
                )
                .eyre(self.db))
            }
        }
    }

    fn give_arguments(
        &mut self,
        table: &bir::Tables,
        argument_places: &[bir::Place],
    ) -> eyre::Result<Vec<Value>> {
        argument_places
            .iter()
            .map(|argument_place| self.give_place(table, *argument_place))
            .collect()
    }

    fn match_labels(
        &self,
        table: &bir::Tables,
        pc: ProgramCounter,
        actual_labels: &[Option<bir::Name>],
        expected_names: &[impl ExpectedName],
    ) -> eyre::Result<()> {
        let db = self.db;

        for (actual_label, expected_name) in actual_labels.iter().zip(expected_names) {
            let expected_name = expected_name.as_word(db);
            if let &Some(actual_label) = actual_label {
                let actual_word = table[actual_label].word;
                if expected_name != actual_word {
                    return Err(error!(
                        self.span_from_bir_name(actual_label),
                        "expected to find an argument named `{}`, but found the name `{}`",
                        expected_name.as_str(db),
                        actual_word.as_str(db),
                    )
                    .eyre(db));
                }
            }
        }

        if actual_labels.len() != expected_names.len() {
            return Err(error!(
                self.span_from_bir(pc.control_point),
                "expected to find {} arguments, but found {}",
                expected_names.len(),
                actual_labels.len(),
            )
            .eyre(db));
        }

        Ok(())
    }
}

trait ExpectedName {
    fn as_word(&self, db: &dyn crate::Db) -> Word;
}

impl ExpectedName for Word {
    fn as_word(&self, _db: &dyn crate::Db) -> Word {
        *self
    }
}

impl ExpectedName for InputTy {
    fn as_word(&self, _db: &dyn crate::Db) -> Word {
        self.name
    }
}
