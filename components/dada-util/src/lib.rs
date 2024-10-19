pub use fxhash::FxHashMap as Map;
pub use fxhash::FxHashSet as Set;
pub use imstr::ImString as Text;

pub type Fallible<T> = anyhow::Result<T>;

pub use anyhow::anyhow;
pub use anyhow::bail;
pub use anyhow::Context;

pub use dada_util_procmacro::*;

#[macro_export]
macro_rules! debug {
    ($($t:tt)*) => {
        eprintln!($($t)*);
    }
}

pub mod typedvec;
pub mod vecset;
