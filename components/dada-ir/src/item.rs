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

impl salsa::DebugWithDb<dyn crate::Db + '_> for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db) -> std::fmt::Result {
        match self {
            Item::Function(v) => std::fmt::Debug::fmt(&v.debug(db), f),
            Item::Class(v) => std::fmt::Debug::fmt(&v.debug(db), f),
        }
    }
}
