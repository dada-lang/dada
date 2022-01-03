use crate::value::Value;
use dada_collections::Map;
use dada_ir::{class::Class, diagnostic::Fallible, word::Word};

#[derive(Debug)]
pub enum Data {
    Instance(Instance),
    Class(Class),
    Tuple(Tuple),
    Bool(bool),
    Uint(u64),
    Int(i64),
    Float(f64),
    String(String),
    None,
}

impl Data {
    pub(crate) fn field(&self, name: Word) -> Fallible<&Value> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Instance {
    pub class: Class,
    pub fields: Map<Word, Value>,
}

#[derive(Debug)]
pub struct Tuple {
    pub fields: Vec<Value>,
}
