use dada_ir::{error, intrinsic::Intrinsic, word::Word};
use eyre::Context;

use crate::{error::DiagnosticBuilderExt, interpreter::Interpreter, value::Value};

pub(crate) type IntrinsicFn = fn(&Interpreter<'_>, Vec<Value>) -> eyre::Result<Value>;

pub(crate) struct IntrinsicDefinition {
    pub(crate) argument_names: Vec<Word>,
    pub(crate) function: IntrinsicFn,
}

impl IntrinsicDefinition {
    pub(crate) fn for_intrinsic(db: &dyn crate::Db, intrinsic: Intrinsic) -> IntrinsicDefinition {
        match intrinsic {
            Intrinsic::Print => IntrinsicDefinition {
                argument_names: vec![Word::from(db, "message")],
                function: intrinsic_write,
            },
        }
    }
}

fn intrinsic_write(interpreter: &Interpreter<'_>, mut values: Vec<Value>) -> eyre::Result<Value> {
    Ok(Value::new(
        interpreter,
        thunk!(async move |interpreter| {
            let message = values.pop().unwrap();
            let message = message.read(interpreter, |data| data.to_word(interpreter))?;
            let message_str = message.as_str(interpreter.db());
            async {
                interpreter.kernel().print(message_str).await?;
                interpreter.kernel().print_newline().await
            }
            .await
            .with_context(|| {
                let span_now = interpreter.span_now();
                error!(span_now, "error printing `{:?}`", message_str).eyre(interpreter.db())
            })?;
            Ok(Value::unit(interpreter))
        }),
    ))
}
