use std::{future::Future, pin::Pin};

use crate::error::DiagnosticBuilderExt;
use crate::intrinsic::IntrinsicDefinition;
use crate::{interpreter::Interpreter, thunk::Thunk, value::Value};
use dada_brew::prelude::*;
use dada_collections::Map;
use dada_ir::word::SpannedOptionalWord;
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
    fn kind_str(&self, interpreter: &Interpreter<'_>) -> String {
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

    fn expected(&self, interpreter: &Interpreter<'_>, what: &str) -> eyre::Report {
        let span = interpreter.span_now();
        error!(
            span,
            "expected {}, found {}",
            what,
            self.kind_str(interpreter)
        )
        .eyre(interpreter.db())
    }

    fn no_such_field(interpreter: &Interpreter<'_>, class: Class, name: Word) -> eyre::Report {
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
        interpreter: &Interpreter<'_>,
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
        interpreter: &Interpreter<'_>,
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

    pub(crate) fn to_bool(&self, interpreter: &Interpreter<'_>) -> eyre::Result<bool> {
        match self {
            Data::Bool(b) => Ok(*b),
            _ => Err(self.expected(interpreter, "a boolean")),
        }
    }

    pub(crate) fn to_word(&self, interpreter: &Interpreter<'_>) -> eyre::Result<Word> {
        match self {
            Data::String(w) => Ok(*w),
            _ => Err(self.expected(interpreter, "a string")),
        }
    }

    pub(crate) fn to_unit(&self, interpreter: &Interpreter<'_>) -> eyre::Result<()> {
        match self {
            Data::Unit(()) => Ok(()),
            _ => Err(self.expected(interpreter, "nothing")),
        }
    }

    pub(crate) fn into_thunk(self, interpreter: &Interpreter<'_>) -> eyre::Result<Thunk> {
        match self {
            Data::Thunk(v) => Ok(v),
            _ => Err(self.expected(interpreter, "an async thunk")),
        }
    }

    pub(crate) fn call<'i>(
        &self,
        interpreter: &'i Interpreter<'_>,
        arguments: Vec<Value>,
        labels: &[SpannedOptionalWord],
    ) -> eyre::Result<DadaFuture<'i>> {
        assert_eq!(arguments.len(), labels.len());
        match self {
            Data::Class(_c) => {
                todo!()
            }
            Data::Function(f) => {
                assert!(arguments.is_empty(), "func argments not yet implemented");
                let bir = f.brew(interpreter.db());
                Ok(interpreter.execute_bir(bir))
            }
            Data::Intrinsic(intrinsic) => {
                let definition = IntrinsicDefinition::for_intrinsic(interpreter.db(), *intrinsic);
                match_labels(interpreter, &labels, &definition.argument_names)?;
                Ok((definition.closure)(interpreter, arguments))
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

fn match_labels(
    interpreter: &Interpreter<'_>,
    actual_labels: &[SpannedOptionalWord],
    expected_names: &[Word],
) -> eyre::Result<()> {
    let db = interpreter.db();

    for (actual_label, expected_name) in actual_labels.iter().zip(expected_names) {
        if let Some(actual_word) = actual_label.word(db) {
            if actual_word != *expected_name {
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
            interpreter.span_now(),
            "expected to find {} arguments, but found {}",
            expected_names.len(),
            actual_labels.len(),
        )
        .eyre(db));
    }

    Ok(())
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
