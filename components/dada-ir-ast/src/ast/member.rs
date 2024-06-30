use salsa::{DebugWithDb, Update};

use super::{Function, VariableDecl};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug, DebugWithDb)]
pub enum Member<'db> {
    Field(VariableDecl<'db>),
    Function(Function<'db>),
}
