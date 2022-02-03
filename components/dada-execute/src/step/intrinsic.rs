use dada_ir::{error, intrinsic::Intrinsic, word::Word};
use eyre::Context;

use crate::{
    error::DiagnosticBuilderExt,
    machine::stringify::DefaultStringify,
    machine::{op::MachineOpExt, Value},
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
                argument_names: vec![Word::from(db, "message")],
                function: |s, v| s.intrinsic_print(v),
                // FIXME: Stepper::intrinsic_write doesn't type check, why?
            },
        }
    }
}

impl Stepper<'_> {
    fn intrinsic_print(&mut self, mut values: Vec<Value>) -> eyre::Result<Value> {
        Ok(self.machine.my_value(thunk!("print", async move |this| {
            let message = values.pop().unwrap();
            this.intrinsic_print_async(message).await
        })))
    }

    pub(super) async fn intrinsic_print_async(&mut self, value: Value) -> eyre::Result<Value> {
        let message_str = DefaultStringify::stringify(&*self.machine, self.db, value);

        async {
            self.kernel.as_mut().unwrap().print(&message_str).await?;
            self.kernel.as_mut().unwrap().print_newline().await
        }
        .await
        .with_context(|| {
            let span_now = self.machine.pc().span(self.db);
            error!(span_now, "error printing `{:?}`", message_str).eyre(self.db)
        })?;

        Ok(self.machine.our_value(()))
    }
}
