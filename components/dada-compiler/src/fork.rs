use std::ops::Deref;

pub struct Fork<C> {
    compiler: C,
}

impl<C> Deref for Fork<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.compiler
    }
}

impl<C> From<C> for Fork<C> {
    fn from(value: C) -> Self {
        Fork { compiler: value }
    }
}
