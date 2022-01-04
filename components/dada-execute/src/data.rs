use crate::{interpreter::Interpreter, value::Value};
use dada_collections::Map;
use dada_ir::{
    class::Class, diagnostic::Fallible, error, func::Function, intrinsic::Intrinsic, word::Word,
};

#[derive(Debug)]
pub(crate) enum Data {
    Instance(Instance),
    Class(Class),
    Function(Function),
    Intrinsic(Intrinsic),
    Tuple(Tuple),
    Bool(bool),
    Uint(u64),
    Int(i64),
    Float(f64),
    String(String),
    None,
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
    Tuple(Tuple),
    Bool(bool),
    Uint(u64),
    Int(i64),
    Float(f64),
    String(String),
}

impl Data {
    pub(crate) fn field(&self, name: Word) -> Fallible<&Value> {
        todo!()
    }

    pub(crate) fn to_bool(&self, interpreter: &Interpreter<'_>) -> Fallible<bool> {
        match self {
            Data::Bool(b) => Ok(*b),
            _ => {
                let span = interpreter.span_now();
                Err(error!(span, "expected a boolean, found something else").emit(interpreter.db()))
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct Instance {
    pub(crate) class: Class,
    pub(crate) fields: Map<Word, Value>,
}

#[derive(Debug)]
pub(crate) struct Tuple {
    pub(crate) fields: Vec<Value>,
}
