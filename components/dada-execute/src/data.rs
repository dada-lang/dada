use std::{future::Future, pin::Pin};

use crate::error::DiagnosticBuilderExt;
use crate::ext::*;
use crate::intrinsic::IntrinsicDefinition;
use crate::{interpreter::Interpreter, thunk::Thunk, value::Value};
use dada_ir::parameter::Parameter;
use dada_ir::word::SpannedOptionalWord;
use dada_ir::{class::Class, error, function::Function, intrinsic::Intrinsic, word::Word};
use dada_parse::prelude::*;

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
    pub(crate) fn kind_str(&self, interpreter: &Interpreter<'_>) -> String {
        let db = interpreter.db();
        match self {
            Data::Instance(i) => format!("an instance of `{}`", i.class.name(db).as_str(db)),
            Data::Class(_) => "a class".to_string(),
            Data::Function(_) => "a function".to_string(),
            Data::Intrinsic(_) => "a function".to_string(),
            Data::Thunk(_) => "a thunk".to_string(),
            Data::Tuple(_) => "a tuple".to_string(),
            Data::Bool(_) => "a boolean".to_string(),
            Data::Uint(_) => "an unsigned integer".to_string(),
            Data::Int(_) => "an integer".to_string(),
            Data::Float(_) => "a float".to_string(),
            Data::String(_) => "a string".to_string(),
            Data::Unit(()) => "nothing".to_string(),
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
        let db = interpreter.db();
        match self {
            Data::Instance(i) => match i.class.field_index(db, name) {
                Some(index) => Ok(&mut i.fields[index]),
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
        let r = self.field_mut(interpreter, name)?;
        *r = value;
        Ok(())
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

    /// This is a bit subtle and probably needs to change. Despite its name,
    /// `call` doesn't actually *call* the value, but rather returns a future
    /// which (when awaited) will perform the call. This is because `call` is
    /// invoked inside of a closure which can't await (which itself maybe should
    /// change!).
    pub(crate) fn call<'i>(
        &self,
        interpreter: &'i Interpreter<'_>,
        arguments: Vec<Value>,
        labels: &[SpannedOptionalWord],
    ) -> eyre::Result<Value> {
        assert_eq!(arguments.len(), labels.len());
        let db = interpreter.db();
        match self {
            Data::Class(c) => {
                let field_names = c.field_names(db);
                match_labels(interpreter, labels, field_names)?;
                let instance = Instance {
                    class: *c,
                    fields: arguments,
                };
                Ok(Value::new(interpreter, instance))
            }
            Data::Function(function) => {
                let parameters = function.parameters(db);
                match_labels(interpreter, labels, parameters)?;
                interpreter.execute_function(*function, arguments)
            }
            Data::Intrinsic(intrinsic) => {
                let definition = IntrinsicDefinition::for_intrinsic(interpreter.db(), *intrinsic);
                match_labels(interpreter, labels, &definition.argument_names)?;
                (definition.function)(interpreter, arguments)
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
    expected_names: &[impl ExpectedName],
) -> eyre::Result<()> {
    let db = interpreter.db();

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
            interpreter.span_now(),
            "expected to find {} arguments, but found {}",
            expected_names.len(),
            actual_labels.len(),
        )
        .eyre(db));
    }

    Ok(())
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

#[derive(Debug)]
pub(crate) struct Instance {
    pub(crate) class: Class,
    pub(crate) fields: Vec<Value>,
}

#[derive(Debug)]
pub(crate) struct Tuple {
    #[allow(dead_code)]
    pub(crate) fields: Vec<Value>,
}
