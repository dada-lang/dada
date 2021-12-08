#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum StorageMode {
    Shared,
    Var,
    Atomic,
}
