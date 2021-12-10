#[salsa::interned(Ty in super::Jar)]
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum TyData {}
