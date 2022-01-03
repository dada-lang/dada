use crate::value::Value;
use dada_collections::Map;
use dada_ir::{class::Class, word::Word};

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Instance {
    pub class: Class,
    pub fields: Map<Word, Value>,
}

#[derive(Clone, Debug)]
pub struct Tuple {
    pub fields: Vec<Value>,
}
