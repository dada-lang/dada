use std::{future::Future, pin::Pin};

use crate::error::DiagnosticBuilderExt;
use crate::intrinsic::IntrinsicDefinition;
use crate::{interpreter::Interpreter, thunk::Thunk, value::Value};
use dada_brew::prelude::*;
use dada_collections::Map;
use dada_ir::{class::Class, error, func::Function, intrinsic::Intrinsic, word::Word};

pub(crate) type DadaFuture<'i> = Pin<Box<dyn Future<Output = eyre::Result<Value>> + 'i>>;

#[derive(Debug)]
pub(crate) enum Data {
    Instance(Instance),
    Class(Class),
    Function(Function),
    Intrinsic(Intrinsic),
    Thunk(Thunk),
    Tuple(Tuple),
    Bool(bool),
    Uint(u64),
    Int(i64),
    Float(f64),
    String(Word),
    Unit(()),
}

macro_rules! data_from_impl {
    ($($variant_name:ident($ty:ty),)*) => {
        $(
            impl From<$ty> for Data {
                fn from(data: $ty) -> Data {
                    Data::$variant_name(data)
                }
            }
        )*
    }
}

data_from_impl! {
    Instance(Instance),
    Class(Class),
    Function(Function),
    Intrinsic(Intrinsic),
    Thunk(Thunk),
    Tuple(Tuple),
    Bool(bool),
    Uint(u64),
    Int(i64),
    Float(f64),
    String(Word),
    Unit(()),
}

impl Data {
    fn kind_str(&self, interpreter: &Interpreter<'_, '_>) -> String {
        let db = interpreter.db();
        match self {
            Data::Instance(i) => format!("an instance of `{}`", i.class.name(db).as_str(db)),
            Data::Class(_) => format!("a class"),
            Data::Function(_) => format!("a function"),
            Data::Intrinsic(_) => format!("a function"),
            Data::Thunk(_) => format!("a thunk"),
            Data::Tuple(_) => format!("a tuple"),
            Data::Bool(_) => format!("a boolean"),
            Data::Uint(_) => format!("an unsigned integer"),
            Data::Int(_) => format!("an integer"),
            Data::Float(_) => format!("a float"),
            Data::String(_) => format!("a string"),
            Data::Unit(()) => format!("nothing"),
        }
    }

    fn expected(&self, interpreter: &Interpreter<'_, '_>, what: &str) -> eyre::Report {
        let span = interpreter.span_now();
        error!(
            span,
            "expected {}, found {}",
            what,
            self.kind_str(interpreter)
        )
        .eyre(interpreter.db())
    }

    fn no_such_field(interpreter: &Interpreter<'_, '_>, class: Class, name: Word) -> eyre::Report {
        let span = interpreter.span_now();
        let class_name = class.name(interpreter.db()).as_str(interpreter.db());
        let class_span = class.name_span(interpreter.db());
        error!(
            span,
            "the class `{}` has no field named `{}`",
            class_name,
            name.as_str(interpreter.db())
        )
        .secondary_label(
            class_span,
            &format!("the class `{}` is declared here", class_name),
        )
        .eyre(interpreter.db())
    }

    pub(crate) fn field_mut(
        &mut self,
        interpreter: &Interpreter<'_, '_>,
        name: Word,
    ) -> eyre::Result<&mut Value> {
        match self {
            Data::Instance(i) => match i.fields.get_mut(&name) {
                Some(value) => Ok(value),
                None => Err(Self::no_such_field(interpreter, i.class, name)),
            },
            _ => Err(self.expected(interpreter, "something with fields")),
        }
    }

    pub(crate) fn assign_field(
        &mut self,
        interpreter: &Interpreter<'_, '_>,
        name: Word,
        value: Value,
    ) -> eyre::Result<()> {
        match self {
            Data::Instance(i) => match i.fields.get_mut(&name) {
                Some(field_value) => Ok(*field_value = value),
                None => Err(Self::no_such_field(interpreter, i.class, name)),
            },
            _ => Err(self.expected(interpreter, "something with fields")),
        }
    }

    pub(crate) fn to_bool(&self, interpreter: &Interpreter<'_, '_>) -> eyre::Result<bool> {
        match self {
            Data::Bool(b) => Ok(*b),
            _ => Err(self.expected(interpreter, "a boolean")),
        }
    }

    pub(crate) fn to_word(&self, interpreter: &Interpreter<'_, '_>) -> eyre::Result<Word> {
        match self {
            Data::String(w) => Ok(*w),
            _ => Err(self.expected(interpreter, "a string")),
        }
    }

    pub(crate) fn to_unit(&self, interpreter: &Interpreter<'_, '_>) -> eyre::Result<()> {
        match self {
            Data::Unit(()) => Ok(()),
            _ => Err(self.expected(interpreter, "nothing")),
        }
    }

    pub(crate) fn into_thunk(self, interpreter: &Interpreter<'_, '_>) -> eyre::Result<Thunk> {
        match self {
            Data::Thunk(v) => Ok(v),
            _ => Err(self.expected(interpreter, "an async thunk")),
        }
    }

    pub(crate) fn call<'i>(
        &self,
        interpreter: &'i Interpreter<'_, '_>,
        named_values: Vec<(Word, Value)>,
    ) -> eyre::Result<DadaFuture<'i>> {
        match self {
            Data::Class(_c) => {
                todo!()
            }
            Data::Function(f) => {
                assert!(named_values.is_empty(), "func argments not yet implemented");
                let bir = f.brew(interpreter.db());
                Ok(interpreter.execute_bir(bir))
            }
            Data::Intrinsic(intrinsic) => {
                let definition = IntrinsicDefinition::for_intrinsic(interpreter.db(), *intrinsic);
                let values = match_values(interpreter, named_values, &definition.argument_names)?;
                Ok((definition.closure)(interpreter, values))
            }
            _ => {
                let span = interpreter.span_now();
                Err(error!(
                    span,
                    "expected something callable, found {}",
                    self.kind_str(interpreter)
                )
                .eyre(interpreter.db()))
            }
        }
    }
}

fn match_values(
    interpreter: &Interpreter<'_, '_>,
    mut named_values: Vec<(Word, Value)>,
    names: &[Word],
) -> eyre::Result<Vec<Value>> {
    let mut values = vec![];
    for name in names {
        if let Some(i) = named_values
            .iter()
            .position(|named_value| named_value.0 == *name)
        {
            let (_, value) = named_values.remove(i);
            values.push(value);
        } else {
            let db = interpreter.db();
            let span_now = interpreter.span_now();
            return Err(error!(
                span_now,
                "expected to find an argument named `{}`, but didn't",
                name.as_str(db)
            )
            .eyre(interpreter.db()));
        }
    }

    if named_values.is_empty() {
        Ok(values)
    } else {
        let db = interpreter.db();
        let span_now = interpreter.span_now();
        let extra_vec: Vec<&str> = named_values
            .iter()
            .map(|(name, _)| name.as_str(db))
            .collect();
        let extra_str = extra_vec.join(", ");
        return Err(error!(span_now, "did not expect argument(s) named `{}`", extra_str).eyre(db));
    }
}

#[derive(Debug)]
pub(crate) struct Instance {
    pub(crate) class: Class,
    pub(crate) fields: Map<Word, Value>,
}

#[derive(Debug)]
pub(crate) struct Tuple {
    #[allow(dead_code)]
    pub(crate) fields: Vec<Value>,
}
