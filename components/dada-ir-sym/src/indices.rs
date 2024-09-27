use salsa::Update;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymBinderIndex(usize);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymBoundVarIndex(usize);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymUniversalVarIndex(usize);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Update, Debug)]
pub struct SymExistentialVarIndex(usize);
