use dada_ir::{error, intrinsic::Intrinsic, word::Word};
use eyre::Context;

use crate::{
    error::DiagnosticBuilderExt,
    machine::stringify::DefaultStringify,
    machine::{op::MachineOpExtMut, ProgramCounter, Value},
    thunk::RustThunk,
};

use super::Stepper;

pub(crate) type IntrinsicFn = fn(&mut Stepper<'_>, Vec<Value>) -> eyre::Result<Value>;

pub(crate) struct IntrinsicDefinition {
    pub(crate) argument_names: Vec<Word>,
    pub(crate) function: IntrinsicFn,
}

impl IntrinsicDefinition {
    pub(crate) fn for_intrinsic(db: &dyn crate::Db, intrinsic: Intrinsic) -> IntrinsicDefinition {
        match intrinsic {
            Intrinsic::Print => IntrinsicDefinition {
                argument_names: vec![Word::intern(db, "message")],
                function: |s, v| s.intrinsic_print(v),
                // FIXME: Stepper::intrinsic_write doesn't type check, why?
            },
        }
    }
}

impl Stepper<'_> {
    /// For intrinsics that yield thunks, when those thunks get awaited,
    /// they invoke this method. This should execute some Rust code and
    /// yield the result. Panics if invoked with an inappropriate intrinsic.
    pub(crate) async fn async_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        mut values: Vec<Value>,
    ) -> eyre::Result<Value> {
        match intrinsic {
            Intrinsic::Print => {
                let value = values.pop().unwrap();
                let await_pc = self.machine.pc();
                self.intrinsic_print_async(await_pc, value).await
            }
        }
    }

    fn intrinsic_print(&mut self, values: Vec<Value>) -> eyre::Result<Value> {
        Ok(self.machine.my_value(
            self.machine.pc(),
            RustThunk::new("print", values, Intrinsic::Print),
        ))
    }

    #[tracing::instrument(level = "Debug", skip(self, await_pc))]
    pub(super) async fn intrinsic_print_async(
        &mut self,
        await_pc: ProgramCounter,
        value: Value,
    ) -> eyre::Result<Value> {
        let message_str = DefaultStringify::stringify_value(&*self.machine, self.db, value);

        async {
            self.kernel
                .as_mut()
                .unwrap()
                .print(await_pc, &message_str)
                .await?;
            self.kernel.as_mut().unwrap().print_newline(await_pc).await
        }
        .await
        .with_context(|| {
            let span_now = self.machine.pc().span(self.db);
            error!(span_now, "error printing `{:?}`", message_str).eyre(self.db)
        })?;

        Ok(self.machine.our_value(await_pc, ()))
    }
}
