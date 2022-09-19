use crate::{class::Class, function::Function, span::FileSpan, word::Word};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Item {
    Function(Function),
    Class(Class),
}

impl Item {
    pub fn span(self, db: &dyn crate::Db) -> FileSpan {
        match self {
            Item::Function(f) => f.span(db).anchor_to(db, f),
            Item::Class(c) => c.span(db).anchor_to(db, c),
        }
    }

    pub fn name(self, db: &dyn crate::Db) -> Word {
        match self {
            Item::Function(f) => f.name(db),
            Item::Class(c) => c.name(db),
        }
    }

    pub fn name_span(self, db: &dyn crate::Db) -> FileSpan {
        match self {
            Item::Function(f) => f.name_span(db).anchor_to(db, f),
            Item::Class(c) => c.name_span(db).anchor_to(db, c),
        }
    }

    pub fn kind_str(self) -> &'static str {
        match self {
            Item::Function(_) => "function",
            Item::Class(_) => "class",
        }
    }
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

impl<Db: ?Sized + crate::Db> salsa::DebugWithDb<Db> for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &Db) -> std::fmt::Result {
        match self {
            Item::Function(v) => std::fmt::Debug::fmt(&v.debug(db), f),
            Item::Class(v) => std::fmt::Debug::fmt(&v.debug(db), f),
        }
    }
}
