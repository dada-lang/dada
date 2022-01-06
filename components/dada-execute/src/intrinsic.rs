use dada_ir::{error, intrinsic::Intrinsic, word::Word};
use eyre::Context;

use crate::{
    data::DadaFuture, error::DiagnosticBuilderExt, interpreter::Interpreter, thunk::Thunk,
    value::Value,
};

pub(crate) type IntrinsicClosure =
    Box<dyn for<'i> Fn(&'i Interpreter<'_>, Vec<Value>) -> DadaFuture<'i>>;

pub(crate) struct IntrinsicDefinition {
    pub(crate) argument_names: Vec<Word>,
    pub(crate) closure: IntrinsicClosure,
}

impl IntrinsicDefinition {
    pub(crate) fn for_intrinsic(db: &dyn crate::Db, intrinsic: Intrinsic) -> Self {
        match intrinsic {
            Intrinsic::Print => IntrinsicDefinition {
                argument_names: vec![Word::from(db, "message")],
                closure: Box::new(|interpreter, values| {
                    Box::pin(intrinsic_write(interpreter, values))
                }),
            },
        }
    }
}

async fn intrinsic_write(
    interpreter: &Interpreter<'_>,
    mut values: Vec<Value>,
) -> eyre::Result<Value> {
    Ok(Value::new(
        interpreter,
        Thunk::new(move |interpreter| {
            Box::pin(async move {
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
            })
        }),
    ))
}
