use crate::{class::Class, func::Function};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Item {
    Function(Function),
    Class(Class),
}

impl From<Function> for Item {
    fn from(value: Function) -> Self {
        Self::Function(value)
    }
}

impl From<Class> for Item {
    fn from(value: Class) -> Self {
        Self::Class(value)
    }
}
