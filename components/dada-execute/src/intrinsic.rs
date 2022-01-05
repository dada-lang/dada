use dada_ir::{intrinsic::Intrinsic, word::Word};

use crate::{data::DadaFuture, interpreter::Interpreter, thunk::Thunk, value::Value};

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
                interpreter.print_bytes(message_str.as_bytes()).await?;
                interpreter.print_bytes("\n".as_bytes()).await?;
                Ok(Value::unit(interpreter))
            })
        }),
    ))
}
