use dada_brew::prelude::*;
use dada_ir::{
    code::{bir, syntax},
    error,
    origin_table::HasOriginIn,
    parameter::Parameter,
    word::{SpannedOptionalWord, Word},
};
use dada_parse::prelude::*;

use crate::{
    error::DiagnosticBuilderExt,
    machine::{op::MachineOpExtMut, Instance, ObjectData, ThunkFn, Value},
    step::intrinsic::IntrinsicDefinition,
};

use super::{IntoSpecifierAndSpan, Stepper};

pub(super) enum CallResult {
    Returned(Value),
    PushedNewFrame,
}

impl Stepper<'_> {
    pub(super) fn call(
        &mut self,
        table: &bir::Tables,
        terminator: bir::Terminator,
        callee: bir::Place,
        argument_places: &[bir::Place],
        labels: &[SpannedOptionalWord],
    ) -> eyre::Result<CallResult> {
        let function_value = self.give_place(table, callee)?;

        assert!(
            self.machine[function_value.permission].valid().is_some(),
            "giving place yielded value with invalid permissions"
        );

        match &self.machine[function_value.object] {
            &ObjectData::Class(c) => {
                let fields = c.fields(self.db);
                self.match_labels(terminator, labels, fields)?;
                let arguments =
                    self.prepare_arguments_for_parameters(table, fields, argument_places)?;
                let instance = Instance {
                    class: c,
                    fields: arguments,
                };
                Ok(CallResult::Returned(self.machine.my_value(instance)))
            }
            &ObjectData::Function(function) => {
                let parameters = function.parameters(self.db);
                self.match_labels(terminator, labels, parameters)?;

                let arguments =
                    self.prepare_arguments_for_parameters(table, parameters, argument_places)?;

                if function.code(self.db).effect.permits_await() {
                    // If the function can await, then it must be an async function.
                    // Now that we have validated the arguments, return a thunk.
                    let thunk = self.machine.my_value(ThunkFn {
                        function,
                        arguments,
                    });
                    Ok(CallResult::Returned(thunk))
                } else {
                    // This is not an async function, so push it onto the stack
                    // and begin execution immediately.
                    let bir = function.brew(self.db);
                    self.machine.push_frame(self.db, bir, arguments);
                    Ok(CallResult::PushedNewFrame)
                }
            }
            &ObjectData::Intrinsic(intrinsic) => {
                let definition = IntrinsicDefinition::for_intrinsic(self.db, intrinsic);
                self.match_labels(callee, labels, &definition.argument_names)?;
                let callee_span = self.span_from_bir(callee);
                let arguments = self.prepare_arguments(
                    table,
                    definition
                        .argument_specifiers
                        .iter()
                        .map(|specifier| (*specifier, callee_span)),
                    argument_places,
                )?;
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

    /// Prepare the arguments according to the given specifiers.
    fn prepare_arguments_for_parameters(
        &mut self,
        table: &bir::Tables,
        parameters: &[Parameter],
        argument_places: &[bir::Place],
    ) -> eyre::Result<Vec<Value>> {
        self.prepare_arguments(
            table,
            parameters
                .iter()
                .map(|parameter| parameter.decl(self.db).specifier),
            argument_places,
        )
    }

    /// Prepare the arguments according to the given specifiers.
    fn prepare_arguments(
        &mut self,
        table: &bir::Tables,
        specifiers: impl Iterator<Item = impl IntoSpecifierAndSpan>,
        argument_places: &[bir::Place],
    ) -> eyre::Result<Vec<Value>> {
        argument_places
            .iter()
            .zip(specifiers)
            .map(|(argument_place, specifier)| {
                self.prepare_value_for_specifier(table, Some(specifier), *argument_place)
            })
            .collect()
    }

    fn match_labels(
        &self,
        call_terminator: impl HasOriginIn<bir::Origins, Origin = syntax::Expr>,
        actual_labels: &[SpannedOptionalWord],
        expected_names: &[impl ExpectedName],
    ) -> eyre::Result<()> {
        let db = self.db;

        for (actual_label, expected_name) in actual_labels.iter().zip(expected_names) {
            let expected_name = expected_name.as_word(db);
            if let Some(actual_word) = actual_label.word(db) {
                if expected_name != actual_word {
                    return Err(error!(
                        actual_label.span(db),
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
                self.span_from_bir(call_terminator),
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

impl ExpectedName for Parameter {
    fn as_word(&self, db: &dyn crate::Db) -> Word {
        self.name(db)
    }
}
